//! Manifest CLI工具二进制入口

use ai_lib::manifest::cli::{Cli, CliRunner};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    CliRunner::run(cli)?;

    Ok(())
}
