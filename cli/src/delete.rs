// SPDX-License-Identifier: MPL-2.0
use crate::{
	args::{CliArgs, CliDelete},
	util::yes_no_prompt,
};
use color_eyre::{eyre::WrapErr, Result};
use owo_colors::OwoColorize;
use zbus::zvariant::OwnedObjectPath;
use zbus_pop_snapshot::{PopSnapshotProxy, SnapshotProxy};

pub async fn delete(args: &CliArgs, delete: &CliDelete) -> Result<()> {
	let connection = zbus::Connection::system()
		.await
		.wrap_err("failed to connect to D-Bus system bus")?;
	let proxy = PopSnapshotProxy::new(&connection)
		.await
		.wrap_err("failed to connect to Pop!_OS snapshot service")?;
	let snapshot_path = match Option::<OwnedObjectPath>::from(
		proxy
			.find_snapshot(&delete.snapshot)
			.await
			.wrap_err("failed to list snapshots")?,
	) {
		Some(path) => path,
		None => {
			println!("Snapshot {} not found", delete.snapshot.blue());
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
			"delete".red(),
			delete.snapshot.blue()
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
			"Alright, {} deleting snapshot {}",
			"not".bold(),
			delete.snapshot.blue()
		);
		return Ok(());
	}

	snapshot
		.delete()
		.await
		.wrap_err_with(|| format!("failed to delete snapshot {}", delete.snapshot))?;

	println!("{} snapshot {}", "Deleted".red(), delete.snapshot.blue());

	Ok(())
}
