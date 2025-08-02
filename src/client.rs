use anyhow::Result;
use colored::*;
use console::style;
use crossterm::{
    cursor::{self, MoveToColumn},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType},
};
use dialoguer::{theme::ColorfulTheme, Select};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::fs;
use std::io::{self, stdout, Write};
use std::path::PathBuf;
use std::{process::Command, time::Duration};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: "mistral".to_string(),
        }
    }
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("commayte");

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }

    config_dir.join("config.toml")
}

fn load_config() -> Config {
    let config_path = get_config_path();

    if let Ok(config_content) = fs::read_to_string(&config_path) {
        if let Ok(config) = toml::from_str(&config_content) {
            return config;
        }
    }

    // Return default config if file doesn't exist or is invalid
    Config::default()
}

fn get_git_diff() -> String {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .stderr(std::process::Stdio::null())
        .output()
        .expect("Failed to get git diff");

    let diff_output = String::from_utf8_lossy(&output.stdout);

    // Files to ignore (dependency managers, lock files, etc.)
    let ignored_patterns = [
        ".lock",
        ".lockfile",
        "package-lock.json",
        "yarn.lock",
        "Cargo.lock",
        "Gemfile.lock",
        "composer.lock",
        "poetry.lock",
        "Pipfile.lock",
        "requirements.txt",
        "package.json",
        "node_modules/",
        "vendor/",
        "target/",
        "dist/",
        "build/",
        ".git/",
        ".DS_Store",
        "*.log",
        "*.tmp",
        "*.cache",
        ".env",
        ".env.local",
        ".env.example",
        "models/",
        "*.gguf",
        "*.bin",
        "*.safetensors",
    ];

    let mut filtered_diff = Vec::new();
    let mut current_file = String::new();
    let mut include_current_file = true;
    let mut total_content_length = 0;
    const MAX_TOTAL_CONTENT: usize = 8000;
    const MAX_FILE_CONTENT: usize = 1000; // Maximum characters per file

    for line in diff_output.lines() {
        // Check if this is a file header (starts with "diff --git")
        if line.starts_with("diff --git") {
            // Process the previous file if it should be included
            if include_current_file && !current_file.is_empty() {
                let mut file_content = current_file.clone();

                // Truncate individual file if it's too large
                if file_content.len() > MAX_FILE_CONTENT {
                    file_content = file_content
                        .chars()
                        .take(MAX_FILE_CONTENT)
                        .collect::<String>();
                    file_content.push_str("\n... (file truncated)");
                }

                let file_size = file_content.len();
                if total_content_length + file_size > MAX_TOTAL_CONTENT {
                    filtered_diff.push("... (diff truncated due to size limit)".to_string());
                    break;
                }
                filtered_diff.push(file_content);
                total_content_length += file_size;
            }

            // Reset for new file
            current_file = line.to_string();
            include_current_file = true;

            // Extract filename from diff header
            if let Some(filename) = extract_filename_from_diff_header(line) {
                // Check if file should be ignored
                let should_ignore = ignored_patterns.iter().any(|pattern| {
                    if let Some(suffix) = pattern.strip_prefix('*') {
                        // Handle wildcard patterns
                        filename.ends_with(suffix)
                    } else if let Some(dir_pattern) = pattern.strip_suffix('/') {
                        // Handle directory patterns
                        filename.starts_with(dir_pattern)
                    } else {
                        // Handle exact patterns
                        filename.contains(pattern)
                    }
                });

                if should_ignore {
                    include_current_file = false;
                }
            }
        } else {
            // Add line to current file if it should be included
            if include_current_file {
                current_file.push('\n');
                current_file.push_str(line);
            }
        }
    }

    // Don't forget the last file
    if include_current_file && !current_file.is_empty() {
        let mut file_content = current_file.clone();

        // Truncate individual file if it's too large
        if file_content.len() > MAX_FILE_CONTENT {
            file_content = file_content
                .chars()
                .take(MAX_FILE_CONTENT)
                .collect::<String>();
            file_content.push_str("\n... (file truncated)");
        }

        let file_size = file_content.len();
        if total_content_length + file_size <= MAX_TOTAL_CONTENT {
            filtered_diff.push(file_content);
        }
    }

    filtered_diff.join("\n\n")
}

fn extract_filename_from_diff_header(header: &str) -> Option<&str> {
    // Extract filename from "diff --git a/filename b/filename" format
    if let Some(start) = header.find("a/") {
        if let Some(end) = header[start + 2..].find(" b/") {
            return Some(&header[start + 2..start + 2 + end]);
        }
    }
    None
}

fn clean_commit_message(message: &str) -> String {
    let first_line = message.lines().next().unwrap_or("").trim();
    let mut cleaned = first_line.to_string();

    // Remove common prefixes and quotes
    for prefix in [
        "commit",
        "Commit:",
        "Commit message:",
        "\"",
        "'",
        "```",
        "`",
    ] {
        cleaned = cleaned.replace(prefix, "");
    }
    cleaned = cleaned.trim().to_string();

    // Validate conventional commit format
    if cleaned.is_empty() {
        return "chore: update code".to_string();
    }

    // Check if it follows type(scope): description format
    if !cleaned.contains(':') {
        return "chore: update code".to_string();
    }

    // Ensure it starts with a valid type
    let valid_types = [
        "feat", "fix", "chore", "docs", "style", "refactor", "test", "perf",
    ];
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() < 2 {
        return "".to_string();
    }

    let type_part = parts[0];
    let has_valid_type = valid_types.iter().any(|&t| type_part.starts_with(t));

    if !has_valid_type {
        return "".to_string();
    }

    cleaned
}

fn generate_commit_message(prompt: &str) -> Result<String> {
    let config = load_config();
    let client = Client::new();

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": config.model,
            "prompt": prompt,
            "options": {
                "temperature": 0.6,
            },
            "system": "You are a concise AI assistant that only returns single-line Git commit messages. Never include explanations.",
            "stream": false
        }))
        .timeout(Duration::from_secs(45))
        .send()?;

    let json: serde_json::Value = response.json()?;
    let raw_msg = json.get("response").and_then(|r| r.as_str()).unwrap_or("");

    Ok(clean_commit_message(raw_msg))
}

fn clear_terminal() {
    io::stdout().flush().unwrap();
}

fn print_header(title: &str) {
    let config = load_config();
    println!("{} {} (using {})\n", "".bold().yellow(), style(title).bold().cyan(), config.model.bold().white());
}

fn edit_in_terminal(initial_text: &str) -> Result<String> {
    // Enable raw mode for better control
    terminal::enable_raw_mode()?;

    // Clear the line and show prompt
    execute!(stdout(), Clear(ClearType::CurrentLine))?;
    execute!(stdout(), Print("Edit commit message: "))?;

    // Print the initial text
    execute!(stdout(), Print(initial_text))?;

    let mut current_text = initial_text.to_string();
    let mut cursor_pos = current_text.len();

    // Position cursor at the end
    execute!(stdout(), MoveToColumn((current_text.len() + 21) as u16))?;

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match code {
                KeyCode::Enter => {
                    execute!(stdout(), Print("\n"))?;
                    break;
                }
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    // Handle Ctrl+C
                    execute!(stdout(), Print("\n"))?;
                    terminal::disable_raw_mode()?;
                    return Err(anyhow::anyhow!("Editing cancelled by user"));
                }
                KeyCode::Backspace => {
                    if cursor_pos > 0 {
                        // Remove character from text
                        current_text.remove(cursor_pos - 1);
                        cursor_pos -= 1;

                        // Move cursor back
                        execute!(stdout(), cursor::MoveLeft(1))?;

                        // Clear the character
                        execute!(stdout(), Print(" "))?;
                        execute!(stdout(), cursor::MoveLeft(1))?;
                    }
                }
                KeyCode::Char(c) => {
                    // Insert character at cursor position
                    current_text.insert(cursor_pos, c);
                    cursor_pos += 1;
                    execute!(stdout(), Print(c))?;
                }
                KeyCode::Left => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                        execute!(stdout(), cursor::MoveLeft(1))?;
                    }
                }
                KeyCode::Right => {
                    if cursor_pos < current_text.len() {
                        cursor_pos += 1;
                        execute!(stdout(), cursor::MoveRight(1))?;
                    }
                }
                KeyCode::Home => {
                    execute!(stdout(), MoveToColumn(20))?;
                    cursor_pos = 0;
                }
                KeyCode::End => {
                    execute!(stdout(), MoveToColumn((current_text.len() + 21) as u16))?;
                    cursor_pos = current_text.len();
                }
                _ => {}
            }
        }
    }

    // Disable raw mode
    terminal::disable_raw_mode()?;

    Ok(current_text)
}

pub fn run() -> Result<()> {
    clear_terminal();

    let diff = get_git_diff();
    if diff.trim().is_empty() {
        println!("{}", "⚠️  No changes to commit.".yellow());
        return Ok(());
    }

    let prompt = format!(
        "Generate a conventional commit message for this git diff.\n\n\
        Look at each file name, added lines (+), and removed lines (-)\n\
        Determine the type based on the changes:\n\
        Types: feat, fix, chore, docs, style, refactor, test, perf\n\
            - feat: new features or functionality\n\
            - fix: bug fixes or error corrections\n\
            - chore: maintenance, dependencies, config changes\n\
            - docs: documentation updates\n\
            - style: formatting, whitespace, code style\n\
            - refactor: code restructuring without changing behavior\n\
            - test: adding or updating tests\n\
        Format: type(scope): description\n\
        Description: short, concise, under 30 characters based on the changes\n\n\
        Examples:\n\
        - feat(ui): add login form\n\
        - fix(api): handle null errors\n\
        - chore(deps): update packages\n\
        - docs(readme): fix typos\n\n\
        Git diff:\n{diff}\n\n\
        Commit message:"
    );

    let mut should_regenerate = true;
    let mut clean_msg = String::new();

    loop {
        if should_regenerate {
            clear_terminal();
            print_header("> Commayte");
            let mut sp = Spinner::new(Spinners::Dots9, "Generating commit message...".into());
            clean_msg = generate_commit_message(&prompt)?;
            sp.stop();
            println!(); // Add newline after spinner
        }

        clear_terminal();

        println!();
        println!(
            "📝 {} {}",
            "Generated commit message:".bold().green(),
            clean_msg.bold().white()
        );
        println!();

        let options = vec![
            "✅ Accept and commit",
            "✏️ Edit message",
            "🔄 Regenerate message",
            "❌ Cancel",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .items(&options)
            .default(0)
            .interact()?;

        let final_message = match selection {
            0 => clean_msg,
            1 => {
                // Edit loop - keep editing until user confirms or cancels
                let mut current_message = clean_msg.clone();

                let edit_result = loop {
                    clear_terminal();
                    print_header("> Commayte");

                    // Use custom in-terminal editing
                    let edited_msg = match edit_in_terminal(&current_message) {
                        Ok(msg) => msg,
                        Err(e) => {
                            if e.to_string().contains("Editing cancelled by user") {
                                // Go back to main menu instead of exiting
                                break None; // Break out of edit loop with None
                            } else {
                                return Err(e);
                            }
                        }
                    };

                    let cleaned_edited_msg = clean_commit_message(&edited_msg);

                    clear_terminal();
                    print_header("> Commayte");

                    println!(
                        "📝 {} {}",
                        "Edited commit message:".bold().green(),
                        cleaned_edited_msg.bold().white()
                    );
                    println!();

                    let confirm_options = vec!["✅ Use this message", "✏️ Edit again", "❌ Cancel"];
                    let confirm_selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Confirm the edited message")
                        .items(&confirm_options)
                        .default(0)
                        .interact()?;

                    match confirm_selection {
                        0 => {
                            // User confirmed, break out of edit loop
                            break Some(cleaned_edited_msg);
                        }
                        1 => {
                            // User wants to edit again, update current message and continue loop
                            current_message = cleaned_edited_msg;
                            continue;
                        }
                        2 => {
                            // User cancelled
                            clear_terminal();
                            print_header("> Commayte");
                            println!("{}", "❌ Cancelled by user".red());
                            return Ok(());
                        }
                        _ => unreachable!(),
                    }
                };

                // Handle the result from the edit loop
                match edit_result {
                    Some(msg) => msg,
                    None => {
                        // Go back to main menu without regenerating
                        should_regenerate = false; // Don't regenerate
                        continue; // This will go back to the main loop
                    }
                }
            }
            2 => {
                should_regenerate = true; // Regenerate next time
                continue; // Regenerate message
            }
            3 => {
                clear_terminal();
                print_header("> Commayte");
                println!("{}", "❌ Cancelled by user".red());
                return Ok(());
            }
            _ => unreachable!(),
        };

        // Commit with the final message (either original, edited, or regenerated)
        clear_terminal();
        print_header("> Commayte");

        let mut commit_sp = Spinner::new(Spinners::Dots9, "Committing changes...".into());

        let commit_result = Command::new("git")
            .args(["commit", "-am", &final_message])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        commit_sp.stop();
        clear_terminal();
        print_header("> Commayte");

        match commit_result {
            Ok(status) if status.success() => {
                println!("{}", "✅ Commit successful!".bold().green());
                println!("📄 Message: {}", final_message.white());
                break;
            }
            Ok(status) => {
                println!("{}", "⚠️ Commit completed with warnings.".yellow());
                println!("📄 Message: {}", final_message.white());
                println!("🔍 Exit code: {}", status.code().unwrap_or(-1));
                break;
            }
            Err(_) => {
                println!("{}", "❌ Git commit failed.".bold().red());
                println!("📄 Message: {}", final_message.white());
                break;
            }
        }
    }

    Ok(())
}
