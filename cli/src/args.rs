use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
	author = "Lucy <lucy@system76.com>",
	about = "CLI tool for managing btrfs snapshots on Pop!_OS"
)]
pub enum CliArgs {
	/// List all snapshots
	List,
	/// Take a snapshot of the current system state
	Create {
		/// The name of the snapshot
		#[clap(short, long)]
		name: Option<String>,
		/// The description of the snapshot
		#[clap(short, long)]
		description: Option<String>,
		/// Which subvolumes to snapshot.
		/// Defaults to everything except for @home.
		#[clap(short, long)]
		subvolumes: Option<Vec<String>>,
	},
	/// Delete an existing snapshot
	Delete {
		/// The UUID of the snapshot to delete
		snapshot: String,
	},
	/// Restore your system to a snapshot
	Restore {
		/// Whether to automatically confirm "yes" to restoring or not.
		#[clap(short, long)]
		yes: bool,
		/// Which subvolumes to snapshot.
		/// Defaults to all subvolumes in the snapshot.
		#[clap(short, long)]
		subvolumes: Option<Vec<String>>,
		/// The UUID of the snapshot to restore
		snapshot: String,
	},
}
