use anyhow::Result;
use colored::*;

use crate::ai;
use crate::config;
use crate::git;
use crate::project;
use crate::prompts;
use crate::system;
use crate::terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run() -> Result<()> {
    terminal::clear_terminal();
    let configuration = config::load_config();

    // Get system specs once at the beginning
    let system_specs = system::get_system_info().unwrap_or(system::SystemSpecs {
        cpu_cores: None,
        cpu_model: None,
        memory_gb: None,
        gpu_model: None,
        os_info: None,
        performance_level: system::PerformanceLevel::Medium,
    });

    terminal::print_header(
        &format!("Using model: {}", configuration.model),
        Some(console::Color::Yellow),
    );

    let diff = git::get_git_diff(&system_specs);
    if diff.trim().is_empty() {
        println!("{}", "⚠️  No changes to commit.".yellow());
        return Ok(());
    }

    let project_context = project::get_project_context();

    let prompt = prompts::generate_commit_prompt(&diff, &project_context);

    let mut should_regenerate = true;
    let mut clean_msg = String::new();

    terminal::print_header(&format!("> Commayte (v{VERSION})"), None);

    loop {
        if should_regenerate {
            terminal::clear_terminal();
            let mut sp = terminal::show_spinner("Generating commit message...");
            clean_msg = ai::generate_commit_message(&prompt, &configuration)?;
            sp.stop();
            println!();
        }

        terminal::clear_terminal();

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
        let selection = terminal::show_selection_menu(options, "What would you like to do?")?;

        let final_message = match selection {
            0 => clean_msg,
            1 => {
                // Edit loop - keep editing until user confirms or cancels
                let mut current_message = clean_msg.clone();

                let edit_result = loop {
                    terminal::clear_terminal();

                    // Use custom in-terminal editing
                    let edited_msg = match terminal::edit_in_terminal(&current_message) {
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

                    let cleaned_edited_msg = ai::clean_commit_message(&edited_msg);

                    terminal::clear_terminal();

                    println!(
                        "📝 {} {}",
                        "Edited commit message:".bold().green(),
                        cleaned_edited_msg.bold().white()
                    );
                    println!();

                    let confirm_options = vec!["✅ Use this message", "✏️ Edit again", "❌ Cancel"];
                    let confirm_selection = terminal::show_selection_menu(
                        confirm_options,
                        "Confirm the edited message",
                    )?;

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
                            terminal::clear_terminal();
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
                terminal::clear_terminal();
                println!("{}", "❌ Cancelled by user".red());
                return Ok(());
            }
            _ => unreachable!(),
        };

        // Commit with the final message (either original, edited, or regenerated)
        terminal::clear_terminal();

        let mut commit_sp = terminal::show_spinner("Committing changes...");

        let commit_result = git::execute_git_commit(&final_message);

        commit_sp.stop();
        terminal::clear_terminal();

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
