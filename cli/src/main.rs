mod args;
mod delete;
mod list;

use self::args::CliArgs;
use clap::Parser;
use color_eyre::{eyre::WrapErr, Result};

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;

	let args = CliArgs::parse();
	match args {
		CliArgs::List => list::list().await.wrap_err("failed to list snapshots"),
		CliArgs::Create {
			name,
			description,
			subvolumes,
		} => todo!(),
		CliArgs::Delete { yes, snapshot } => delete::delete(yes, snapshot)
			.await
		.wrap_err("failed to delete snapshot"),
		CliArgs::Restore {
			yes,
			subvolumes,
			snapshot,
		} => todo!(),
	}
}
