// SPDX-License-Identifier: MPL-2.0
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(
	author = "Lucy <lucy@system76.com>",
	about = "CLI tool for managing btrfs snapshots on Pop!_OS"
)]
pub struct CliArgs {
	/// Whether to automatically confirm "yes" to prompts or not.
	#[clap(short, long)]
	pub yes: bool,
	#[clap(subcommand)]
	pub subcommand: CliSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum CliSubcommand {
	/// List all snapshots.
	List,
	/// Take a snapshot of the current system state.
	Create(CliCreate),
	/// Delete an existing snapshot
	Delete(CliDelete),
	/// Restore your system to a snapshot.
	Restore(CliRestore),
}

#[derive(Debug, Args)]
pub struct CliCreate {
	#[clap(short, long)]
	pub name: Option<String>,
	/// The description of the snapshot
	#[clap(short, long)]
	pub description: Option<String>,
	/// Which subvolumes to snapshot.
	/// Defaults to everything except for @home.
	#[clap(short, long)]
	pub subvolumes: Option<Vec<String>>,
}

#[derive(Debug, Args)]
pub struct CliDelete {
	/// The UUID of the snapshot to delete.
	#[clap(short, long)]
	pub snapshot: String,
}

#[derive(Debug, Args)]
pub struct CliRestore {
	/// Which subvolumes to snapshot.
	/// Defaults to all subvolumes in the snapshot.
	#[clap(short, long)]
	pub subvolumes: Option<Vec<String>>,
	/// The UUID of the snapshot to restore
	pub snapshot: String,
}
