use anyhow::Result;
use console::style;
use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType},
};
use dialoguer::{theme::ColorfulTheme, Select};
use spinners::{Spinner, Spinners};
use std::io::{self, stdout, Write};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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

pub fn edit_in_terminal(initial_text: &str) -> Result<String> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::CurrentLine))?;

    let header_title = "Edit commit message: ";
    execute!(stdout, Print(header_title))?;

    let mut graphemes: Vec<String> = UnicodeSegmentation::graphemes(initial_text, true)
        .map(|g| g.to_string())
        .collect();
    let mut cursor_pos = graphemes.len();

    // Print the initial text
    for g in &graphemes {
        execute!(stdout, Print(g))?;
    }

    execute!(
        stdout,
        MoveToColumn(
            (header_title.len() + graphemes.iter().map(|g| g.width()).sum::<usize>()) as u16
        )
    )?;

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match code {
                KeyCode::Enter => {
                    execute!(stdout, Print("\n"))?;
                    break;
                }
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    execute!(stdout, Print("\n"))?;
                    terminal::disable_raw_mode()?;
                    return Err(anyhow::anyhow!("Editing cancelled by user"));
                }
                KeyCode::Backspace => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                        graphemes.remove(cursor_pos);

                        // Redraw line
                        execute!(
                            stdout,
                            MoveToColumn(header_title.len() as u16),
                            Clear(ClearType::UntilNewLine)
                        )?;
                        for g in &graphemes {
                            execute!(stdout, Print(g))?;
                        }

                        // Move cursor
                        let left_width = graphemes[..cursor_pos]
                            .iter()
                            .map(|g| g.width())
                            .sum::<usize>() as u16;
                        execute!(
                            stdout,
                            MoveToColumn((header_title.len() + left_width as usize) as u16)
                        )?;
                    }
                }
                KeyCode::Char(c) => {
                    graphemes.insert(cursor_pos, c.to_string());
                    cursor_pos += 1;

                    // Redraw line
                    execute!(
                        stdout,
                        MoveToColumn(header_title.len() as u16),
                        Clear(ClearType::UntilNewLine)
                    )?;
                    for g in &graphemes {
                        execute!(stdout, Print(g))?;
                    }

                    // Move cursor
                    let left_width = graphemes[..cursor_pos]
                        .iter()
                        .map(|g| g.width())
                        .sum::<usize>() as u16;
                    execute!(
                        stdout,
                        MoveToColumn((header_title.len() + left_width as usize) as u16)
                    )?;
                }
                KeyCode::Left => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                        let left_width = graphemes[..cursor_pos]
                            .iter()
                            .map(|g| g.width())
                            .sum::<usize>() as u16;
                        execute!(
                            stdout,
                            MoveToColumn((header_title.len() + left_width as usize) as u16)
                        )?;
                    }
                }
                KeyCode::Right => {
                    if cursor_pos < graphemes.len() {
                        cursor_pos += 1;
                        let left_width = graphemes[..cursor_pos]
                            .iter()
                            .map(|g| g.width())
                            .sum::<usize>() as u16;
                        execute!(
                            stdout,
                            MoveToColumn((header_title.len() + left_width as usize) as u16)
                        )?;
                    }
                }
                KeyCode::Home => {
                    cursor_pos = 0;
                    execute!(stdout, MoveToColumn(header_title.len() as u16))?;
                }
                KeyCode::End => {
                    cursor_pos = graphemes.len();
                    let width = graphemes.iter().map(|g| g.width()).sum::<usize>() as u16;
                    execute!(
                        stdout,
                        MoveToColumn((header_title.len() + width as usize) as u16)
                    )?;
                }
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode()?;

    let final_string = graphemes.concat().trim().to_string();
    Ok(final_string)
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
