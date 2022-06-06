// SPDX-License-Identifier: MPL-2.0
use crate::{
	args::{CliArgs, CliCreate},
	util::yes_no_prompt,
};
use color_eyre::{eyre::WrapErr, Result};
use owo_colors::OwoColorize;
use zbus_pop_snapshot::{PopSnapshotProxy, SnapshotProxy};

pub async fn create(args: &CliArgs, create: &CliCreate) -> Result<()> {
	let connection = zbus::Connection::system()
		.await
		.wrap_err("failed to connect to D-Bus system bus")?;
	let proxy = PopSnapshotProxy::new(&connection)
		.await
		.wrap_err("failed to connect to Pop!_OS snapshot service")?;
	let is_sure = args.yes || {
		println!(
			"Are you {} you want to {} a snapshot?",
			"SURE".bold(),
			"create".green()
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
		println!("Alright, {} creating a snapshot.", "not".red().bold());
		return Ok(());
	}

	print!("Creating a new snapshot, ");
	match &create.subvolumes {
		Some(subvolumes) => {
			let mut iter = subvolumes.iter().peekable();
			print!("{}: ", "using the following subvolumes".dimmed());
			while let Some(subvolume) = iter.next() {
				print!("{}", subvolume.green());
				if iter.peek().is_some() {
					print!(", ");
				} else {
					println!()
				}
			}
		}
		None => {
			println!("using {}", "all valid subvolumes".green().bold());
		}
	}

	let new_snapshot_path = proxy
		.create_snapshot(
			create.name.clone().into(),
			create.description.clone().into(),
			create.subvolumes.clone().into(),
		)
		.await
		.wrap_err("failed to create snapshot")?;
	let new_snapshot = SnapshotProxy::builder(&connection)
		.path(&new_snapshot_path)
		.wrap_err("failed to connect to new snapshot path")?
		.build()
		.await
		.wrap_err("failed to connect to new snapshot")?;
	let uuid = new_snapshot
		.uuid()
		.await
		.wrap_err("failed to get snapshot UUID")?;
	let name = new_snapshot
		.name()
		.await
		.wrap_err("failed to get snapshot name")?;
	let description = new_snapshot
		.description()
		.await
		.wrap_err("failed to get snapshot description")?;
	let subvolumes = new_snapshot
		.subvolumes()
		.await
		.wrap_err("failed to get snapshot subvolumes")?;

	println!("Created snapshot {}", uuid.blue());
	println!(
		"\t{}: {}",
		"Name".bold(),
		if name.is_empty() {
			"none set"
		} else {
			name.as_str()
		}
		.dimmed(),
	);
	println!(
		"\t{}: {}",
		"Description".bold(),
		if description.is_empty() {
			"none set"
		} else {
			description.as_str()
		}
		.dimmed(),
	);
	print!("\t{}: ", "Subvolumes".bold());
	let mut iter = subvolumes.iter().peekable();
	while let Some(subvolume) = iter.next() {
		print!("{}", subvolume.green());
		if iter.peek().is_some() {
			print!(", ");
		} else {
			println!()
		}
	}

	Ok(())
}
