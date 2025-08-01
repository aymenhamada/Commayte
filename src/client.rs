use std::{process::Command, time::Duration};
use anyhow::Result;
use reqwest::blocking::Client;
use spinners::{Spinner, Spinners};
use dialoguer::{theme::ColorfulTheme, Select};
use std::io::{self, Write, BufRead, stdin, stdout, Read};
use colored::*;
use console::style;

fn get_git_diff() -> String {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .stderr(std::process::Stdio::null())
        .output()
        .expect("Failed to get git diff");
    
    let diff_output = String::from_utf8_lossy(&output.stdout);
    
    // Files to ignore (dependency managers, lock files, etc.)
    let ignored_patterns = [
        ".lock", ".lockfile", "package-lock.json", "yarn.lock", "Cargo.lock", "Gemfile.lock",
        "composer.lock", "poetry.lock", "Pipfile.lock", "requirements.txt", "package.json",
        "node_modules/", "vendor/", "target/", "dist/", "build/", ".git/", ".DS_Store",
        "*.log", "*.tmp", "*.cache", ".env", ".env.local", ".env.example",
        "models/", "*.gguf", "*.bin", "*.safetensors"
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
                    file_content = file_content.chars().take(MAX_FILE_CONTENT).collect::<String>();
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
                    if pattern.starts_with('*') {
                        // Handle wildcard patterns
                        let suffix = &pattern[1..];
                        let matches = filename.ends_with(suffix);
                        matches
                    } else if pattern.ends_with('/') {
                        // Handle directory patterns
                        let dir_pattern = &pattern[..pattern.len()-1];
                        let matches = filename.starts_with(dir_pattern);
                        matches
                    } else {
                        // Handle exact patterns
                        let matches = filename.contains(pattern);
                        matches
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
            file_content = file_content.chars().take(MAX_FILE_CONTENT).collect::<String>();
            file_content.push_str("\n... (file truncated)");
        }
        
        let file_size = file_content.len();
        if total_content_length + file_size <= MAX_TOTAL_CONTENT {
            filtered_diff.push(file_content);
        }
    }

    let result = filtered_diff.join("\n\n");
    result
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
    for prefix in ["commit", "Commit:", "Commit message:", "\"", "'", "```", "`"] {
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
    let valid_types = ["feat", "fix", "chore", "docs", "style", "refactor", "test", "perf"];
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() < 2 {
        return "chore: update code".to_string();
    }

    let type_part = parts[0];
    let has_valid_type = valid_types.iter().any(|&t| type_part.starts_with(t));
    
    if !has_valid_type {
        return "chore: update code".to_string();
    }

    cleaned
}

fn generate_commit_message(prompt: &str) -> Result<String> {
    let client = Client::new();
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "mistral",
            "prompt": prompt,
            "max_tokens": 100,
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
    println!(
        "{} {}\n",
        "".bold().yellow(),
        style(title).bold().cyan()
    );
}

fn edit_in_terminal(initial_text: &str) -> Result<String> {
    // For now, let's use a simpler approach that's more reliable
    // Show the current message and let user type a new one
    // This is more reliable than trying to implement complex terminal editing
    println!();
    println!("{}", "Current message:".dimmed());
    println!("{}", initial_text.bold().white());
    println!();
    println!("{}", "Type the new message (or press Enter to keep current):".yellow());
    
    let mut input = String::new();
    stdin().lock().read_line(&mut input)?;
    
    let edited = input.trim();
    if edited.is_empty() {
        Ok(initial_text.to_string())
    } else {
        Ok(edited.to_string())
    }
}

pub fn run() -> Result<()> {
    clear_terminal();

    let diff = get_git_diff();
    if diff.trim().is_empty() {
        println!("{}", "⚠️  No changes to commit.".yellow());
        return Ok(());
    }

    let prompt = format!(
        "Analyze the git diff below and generate a conventional commit message.\n\n\
        Instructions:\n\
        1. Look at each file name, added lines (+), and removed lines (-)\n\
        2. Determine the type based on the changes:\n\
           - feat: new features or functionality\n\
           - fix: bug fixes or error corrections\n\
           - chore: maintenance, dependencies, config changes\n\
           - docs: documentation updates\n\
           - style: formatting, whitespace, code style\n\
           - refactor: code restructuring without changing behavior\n\
           - test: adding or updating tests\n\
           - perf: performance improvements\n\
        3. Determine scope from the file path (e.g., client, server, config, ui)\n\
        4. Write a description based on what was actually changed\n\
        5. Use format: type(scope): description\n\
        6. Keep description short concise and under 30 characters\n\
        7. Return ONLY the commit message\n\n\
        RETURN ONLY THE COMMIT MESSAGE, NOTHING ELSE.\n\n\
        RESPECT CONVENTIONAL COMMIT SPECIFICATION.\n\n\
        RETURN ONLY THE COMMIT MESSAGE, NOTHING ELSE.\n\n\
        Git diff:\n{diff}\n\n\
        Commit message:"
    );

    loop {
        let mut sp = Spinner::new(Spinners::Dots9, "Generating commit message...".into());
        let clean_msg = generate_commit_message(&prompt)?;
        sp.stop();

        clear_terminal();
        print_header("> Commayte");

        println!("📝 {} {}", "Generated commit message:".bold().green(), clean_msg.bold().white());
        println!();

        let options = vec!["✅ Accept and commit", "✏️ Edit message", "🔄 Regenerate message", "❌ Cancel"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .items(&options)
            .default(0)
            .interact()?;

        let final_message = match selection {
            0 => clean_msg,
            1 => {
                clear_terminal();
                print_header("> Commayte");
                
                // Use custom in-terminal editing
                let edited_msg = edit_in_terminal(&clean_msg)?;
                
                let cleaned_edited_msg = clean_commit_message(&edited_msg);
                
                clear_terminal();
                print_header("> Commayte");
                
                println!("📝 {} {}", "Edited commit message:".bold().green(), cleaned_edited_msg.bold().white());
                println!();
                
                let confirm_options = vec!["✅ Use this message", "✏️ Edit again", "❌ Cancel"];
                let confirm_selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Confirm the edited message")
                    .items(&confirm_options)
                    .default(0)
                    .interact()?;
                
                match confirm_selection {
                    0 => cleaned_edited_msg,
                    1 => continue, // This will restart the edit loop
                    2 => {
                        clear_terminal();
                        print_header("> Commayte");
                        println!("{}", "❌ Cancelled by user".red());
                        return Ok(());
                    }
                    _ => unreachable!(),
                }
            }
            2 => continue, // Regenerate message
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
