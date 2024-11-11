use patinator::{config::Config, runner::VersionChecker};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "version-tracker")]
#[command(about = "Track versions across multiple services and environments")]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::from_file(&cli.config)?;
    let checker = VersionChecker::new(config);
    checker.check_all().await?;
    Ok(())
}
