use color_eyre::{eyre::WrapErr, Result};
use owo_colors::OwoColorize;
use std::io::Read;
use zbus::zvariant::OwnedObjectPath;
use zbus_pop_snapshot::{PopSnapshotProxy, SnapshotProxy};

pub async fn delete(yes: bool, snapshot_uuid: String) -> Result<()> {
	let connection = zbus::Connection::system()
		.await
		.wrap_err("failed to connect to D-Bus system bus")?;
	let proxy = PopSnapshotProxy::new(&connection)
		.await
		.wrap_err("failed to connect to Pop!_OS snapshot service")?;
	let snapshot_path = match Option::<OwnedObjectPath>::from(
		proxy
			.find_snapshot(&snapshot_uuid)
			.await
			.wrap_err("failed to list snapshots")?,
	) {
		Some(path) => path,
		None => {
			println!("Snapshot {} not found", snapshot_uuid.blue());
			return Ok(());
		}
	};

	let snapshot = SnapshotProxy::builder(&connection)
		.path(&snapshot_path)
		.wrap_err_with(|| format!("failed to connect to snapshot {}", snapshot_path.as_str()))?
		.build()
		.await
		.wrap_err_with(|| format!("failed to connect to snapshot {}", snapshot_path.as_str()))?;

	let is_sure = yes || {
		println!(
			"Are you {} you want to {} snapshot {}?",
			"SURE".bold(),
			"delete".red(),
			snapshot_uuid.blue()
		);
		println!(
			"Press '{}' for {}, or any other key to {}",
			"y".green().bold(),
			"yes".green(),
			"cancel".red()
		);
		let mut buf = [0_u8];
		std::io::stdin()
			.lock()
			.read_exact(&mut buf)
			.wrap_err("failed to read from stdin")?;
		let key = buf[0] as char;
		key == 'Y' || key == 'y'
	};
	if !is_sure {
		println!(
			"Alright, {} deleting snapshot {}",
			"not".bold(),
			snapshot_uuid.blue()
		);
		return Ok(());
	}

	snapshot
		.delete()
		.await
		.wrap_err_with(|| format!("failed to delete snapshot {snapshot_uuid}"))?;

	println!("{} snapshot {}", "Deleted".red(), snapshot_uuid.blue());

	Ok(())
}
