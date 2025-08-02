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
use spinners::{Spinner, Spinners};
use std::io::{self, stdout, Write};
use std::process::Command;

use crate::config;

/// Clears the terminal screen
pub fn clear_terminal() {
    io::stdout().flush().unwrap();
}

/// Prints the application header with model information
pub fn print_header(title: &str) {
    let configuration = config::load_config();
    println!(
        "{} {} (using {})\n",
        "".bold().yellow(),
        style(title).bold().cyan(),
        configuration.model.bold().white()
    );
}

/// Provides in-terminal editing functionality for commit messages
pub fn edit_in_terminal(initial_text: &str) -> Result<String> {
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

/// Shows a selection menu and returns the user's choice
pub fn show_selection_menu(options: Vec<&str>, prompt: &str) -> Result<usize> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&options)
        .default(0)
        .interact()?;

    Ok(selection)
}

/// Shows a spinner with the given message
pub fn show_spinner(message: &str) -> Spinner {
    Spinner::new(Spinners::Dots9, message.into())
}

/// Executes a git commit command
pub fn execute_git_commit(message: &str) -> Result<std::process::ExitStatus> {
    let result = Command::new("git")
        .args(["commit", "-am", message])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    Ok(result)
}
