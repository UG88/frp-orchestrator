use anyhow::Result;
use clap::Parser;
use frpctl::commands::{run_cli, Cli};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Err(e) = run_cli(cli).await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
    Ok(())
}
