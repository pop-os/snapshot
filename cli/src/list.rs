use color_eyre::{eyre::WrapErr, Result};
use owo_colors::OwoColorize;
use zbus_pop_snapshot::{PopSnapshotProxy, SnapshotProxy};

pub async fn list() -> Result<()> {
	let connection = zbus::Connection::system()
		.await
		.wrap_err("failed to connect to D-Bus system bus")?;
	let proxy = PopSnapshotProxy::new(&connection)
		.await
		.wrap_err("failed to connect to Pop!_OS snapshot service")?;
	let snapshot_objects = proxy
		.snapshots()
		.await
		.wrap_err("failed to list snapshots")?;
	for snapshot_path in snapshot_objects {
		// We don't use ? here, as we want to gracefully handle a snapshot not existing for some reason.
		let snapshot = match SnapshotProxy::builder(&connection).path(&snapshot_path) {
			Ok(snapshot) => snapshot,
			Err(err) => {
				println!(
					"{} to get info for the snapshot object {}:\n\t{}",
					"Failed".red(),
					snapshot_path.as_str().blue(),
					err.red()
				);
				continue;
			}
		};
		let snapshot = match snapshot.build().await {
			Ok(snapshot) => snapshot,
			Err(err) => {
				println!(
					"{} to get info for the snapshot object {}:\n\t{}",
					"Failed".red(),
					snapshot_path.as_str().blue(),
					err.red()
				);
				continue;
			}
		};
		let uuid = snapshot
			.uuid()
			.await
			.wrap_err("failed to get snapshot UUID")?;
		println!("Snapshot {}", uuid.green());
		let name = snapshot
			.name()
			.await
			.wrap_err("failed to get snapshot name")?;
		let description = snapshot
			.description()
			.await
			.wrap_err("failed to get snapshot description")?;
		let subvolumes = snapshot
			.subvolumes()
			.await
			.wrap_err("failed to get snapshot subvolumes")?;
		if !name.is_empty() {
			println!("\t{}: {}", "Name".bold(), name.dimmed());
		}
		if !description.is_empty() {
			println!("\t{}: {}", "Description".bold(), description.dimmed());
		}
		print!("\t{}: ", "Subvolumes".bold());
		let mut subvolumes = subvolumes.iter().peekable();
		while let Some(subvolume) = subvolumes.next() {
			print!("{} ", subvolume.green());
			match subvolumes.peek() {
				Some(_) => print!(", "),
				None => println!(),
			}
		}
	}

	Ok(())
}
