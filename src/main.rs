mod client;

fn main() -> anyhow::Result<()> {
    client::run()?;

    Ok(())
}