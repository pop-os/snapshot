mod args;

use clap::Parser;
use color_eyre::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;

	let args = args::CliArgs::parse();

	Ok(())
}
