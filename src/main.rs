mod ai;
mod client;
mod config;
mod git;
mod project;
mod prompts;
mod terminal;

fn main() -> anyhow::Result<()> {
    client::run()?;

    Ok(())
}
