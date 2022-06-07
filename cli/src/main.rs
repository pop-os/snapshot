// SPDX-License-Identifier: MPL-2.0
mod args;
mod create;
mod delete;
mod list;
mod restore;
pub(crate) mod util;

use self::args::{CliArgs, CliSubcommand};
use clap::Parser;
use color_eyre::{eyre::WrapErr, Result};

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;

	let args = CliArgs::parse();
	match &args.subcommand {
		CliSubcommand::List => list::list().await.wrap_err("failed to list snapshots"),
		CliSubcommand::Create(create) => create::create(&args, create)
			.await
			.wrap_err("failed to create snapshot"),
		CliSubcommand::Delete(delete) => delete::delete(&args, delete)
			.await
			.wrap_err("failed to delete snapshot"),
		CliSubcommand::Restore(restore) => restore::restore(&args, restore)
			.await
			.wrap_err("failed to restore snapshot"),
	}
}
