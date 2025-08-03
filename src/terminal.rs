use anyhow::Result;
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

/// Clears the terminal screen
pub fn clear_terminal() {
    io::stdout().flush().unwrap();
}

/// Prints the application header with model information
pub fn print_header(title: &str, color: Option<console::Color>) {
    if let Some(color) = color {
        match color {
            console::Color::Red => println!("{}\n", style(title).bold().red()),
            console::Color::Green => println!("{}\n", style(title).bold().green()),
            console::Color::Yellow => println!("{}\n", style(title).bold().yellow()),
            console::Color::Blue => println!("{}\n", style(title).bold().blue()),
            console::Color::Magenta => println!("{}\n", style(title).bold().magenta()),
            console::Color::Cyan => println!("{}\n", style(title).bold().cyan()),
            console::Color::White => println!("{}\n", style(title).bold().white()),
            console::Color::Black => println!("{}\n", style(title).bold().black()),
            _ => println!("{}\n", style(title).bold().cyan()),
        }
    } else {
        println!("{}\n", style(title).bold().cyan());
    }
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

                        // Redraw the rest of the text from cursor position
                        let remaining_text = &current_text[cursor_pos..];
                        execute!(stdout(), Print(remaining_text))?;
                        execute!(stdout(), Print(" "))?; // Clear the last character

                        // Move cursor back to the correct position
                        execute!(
                            stdout(),
                            cursor::MoveLeft((remaining_text.len() + 1) as u16)
                        )?;
                    }
                }
                KeyCode::Char(c) => {
                    // Insert character at cursor position
                    current_text.insert(cursor_pos, c);
                    cursor_pos += 1;
                    execute!(stdout(), Print(c))?;

                    // Redraw the rest of the text from the new cursor position
                    let remaining_text = &current_text[cursor_pos..];
                    execute!(stdout(), Print(remaining_text))?;

                    // Move cursor back to the correct position (after the inserted character)
                    if !remaining_text.is_empty() {
                        execute!(stdout(), cursor::MoveLeft(remaining_text.len() as u16))?;
                    }
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
