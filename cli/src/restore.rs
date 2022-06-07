// SPDX-License-Identifier: MPL-2.0
use crate::{
	args::{CliArgs, CliRestore},
	util::yes_no_prompt,
};
use color_eyre::{eyre::WrapErr, Result};
use owo_colors::OwoColorize;
use zbus::zvariant::OwnedObjectPath;
use zbus_pop_snapshot::{PopSnapshotProxy, SnapshotProxy};

pub async fn restore(args: &CliArgs, restore: &CliRestore) -> Result<()> {
	let connection = zbus::Connection::system()
		.await
		.wrap_err("failed to connect to D-Bus system bus")?;
	let proxy = PopSnapshotProxy::new(&connection)
		.await
		.wrap_err("failed to connect to Pop!_OS snapshot service")?;
	let snapshot_path = match Option::<OwnedObjectPath>::from(
		proxy
			.find_snapshot(&restore.snapshot)
			.await
			.wrap_err("failed to list snapshots")?,
	) {
		Some(path) => path,
		None => {
			println!("Snapshot {} not found", restore.snapshot.blue());
			return Ok(());
		}
	};

	let snapshot = SnapshotProxy::builder(&connection)
		.path(&snapshot_path)
		.wrap_err_with(|| format!("failed to connect to snapshot {}", snapshot_path.as_str()))?
		.build()
		.await
		.wrap_err_with(|| format!("failed to connect to snapshot {}", snapshot_path.as_str()))?;

	let is_sure = args.yes || {
		println!(
			"Are you {} you want to {} snapshot {}?",
			"SURE".bold(),
			"restore".green(),
			restore.snapshot.blue()
		);
		println!(
			"Press '{}' for {}, or any other key to {}",
			"y".green().bold(),
			"yes".green(),
			"cancel".red()
		);
		yes_no_prompt()
	};
	if !is_sure {
		println!(
			"Alright, {} restoring to snapshot {}",
			"not".bold(),
			restore.snapshot.blue()
		);
		return Ok(());
	}

	snapshot
		.restore()
		.await
		.wrap_err_with(|| format!("failed to restore snapshot {}", restore.snapshot))?;

	println!(
		"Snapshot {} has been restored. You should {} your system, as any changes from now to restored subvolumes will be lost.",
		restore.snapshot.blue(),
		"reboot".bold()
	);

	Ok(())
}
