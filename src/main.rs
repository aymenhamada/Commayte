mod ai;
mod client;
mod config;
mod git;
mod project;
mod prompts;
mod system;
mod terminal;
mod update;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "commayte")]
#[command(about = "AI-powered git commit message generator with interactive CLI")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Add emojis to commit messages for better visual appeal
    #[arg(long)]
    emoji: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for updates and optionally perform auto-update
    Update,
    /// Show version information
    Version,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Update) => {
            match update::check_for_updates()? {
                Some(release) => {
                    println!();
                    println!("📝 Release notes:");
                    println!("{}", release.body);
                    println!();

                    let should_update = dialoguer::Confirm::new()
                        .with_prompt("Would you like to update now?")
                        .default(true)
                        .interact()?;

                    if should_update {
                        update::perform_update(&release)?;
                    } else {
                        println!("⏭️  Update skipped. You can run 'commayte update --auto' later.");
                    }
                }
                None => {
                    // Already up to date
                }
            }
        }
        Some(Commands::Version) => {
            update::show_update_info();
        }
        None => {
            // Default behavior - run the commit message generator
            client::run(cli.emoji)?;
        }
    }

    Ok(())
}
