// SPDX-License-Identifier: MPL-2.0

use crate::{
	create_new_snapshot,
	snapshot::{metadata::SnapshotMetadata, MountedBtrfs},
};
use anyhow::{Context, Result};
use std::{collections::HashSet, sync::Arc};
use time::format_description::well_known::Rfc3339;
use tokio::sync::RwLock;
use zbus::{dbus_interface, zvariant::OwnedObjectPath, MessageHeader, ObjectServer};

pub struct SnapshotObject {
	metadata: SnapshotMetadata,
	snapshots: Arc<RwLock<HashSet<OwnedObjectPath>>>,
}

impl SnapshotObject {
	pub(crate) fn new(
		metadata: SnapshotMetadata,
		snapshots: Arc<RwLock<HashSet<OwnedObjectPath>>>,
	) -> Self {
		Self {
			metadata,
			snapshots,
		}
	}
}

impl SnapshotObject {
	async fn update_metadata_file(&self) -> Result<()> {
		let btrfs = MountedBtrfs::new().await.context("failed to mount btrfs")?;
		let metadata_path = btrfs
			.path()
			.join("@snapshots/pop-snapshots")
			.join(self.metadata.uuid.to_string())
			.with_extension("snapshot.json");
		tokio::fs::write(
			&metadata_path,
			serde_json::to_string_pretty(&self.metadata)?,
		)
		.await
		.with_context(|| {
			format!(
				"failed to write updated metadata to file {}",
				metadata_path.display()
			)
		})?;
		Ok(())
	}
}

#[dbus_interface(name = "com.system76.SnapshotDaemon.Snapshot")]
impl SnapshotObject {
	#[dbus_interface(property)]
	async fn creation_time(&self) -> String {
		self.metadata
			.creation_time
			.format(&Rfc3339)
			.expect("failed to format time as RFC 3399")
	}

	#[dbus_interface(property)]
	async fn name(&self) -> String {
		self.metadata.name.clone().unwrap_or_default()
	}

	#[dbus_interface(property)]
	async fn set_name(&mut self, value: &str) {
		self.metadata.name = if value.trim().is_empty() {
			None
		} else {
			Some(value.to_owned())
		};
		self.update_metadata_file()
			.await
			.expect("failed to update metadata file");
	}

	#[dbus_interface(property)]
	async fn description(&self) -> String {
		self.metadata.description.clone().unwrap_or_default()
	}

	#[dbus_interface(property)]
	async fn set_description(&mut self, value: &str) {
		self.metadata.description = if value.trim().is_empty() {
			None
		} else {
			Some(value.to_owned())
		};
		self.update_metadata_file()
			.await
			.expect("failed to update metadata file");
	}

	#[dbus_interface(property)]
	async fn subvolumes(&self) -> Vec<String> {
		self.metadata.subvolumes.clone()
	}

	#[dbus_interface(property)]
	async fn uuid(&self) -> String {
		self.metadata.uuid.to_string()
	}

	async fn restore(&self, #[zbus(object_server)] object_server: &ObjectServer) {
		let btrfs = MountedBtrfs::new().await.expect("failed to mount btrfs");
		let new_snapshot = btrfs
			.restore_snapshot(&self.metadata)
			.await
			.expect("failed to restore snapshot");
		let new_snapshot_object = SnapshotObject::new(new_snapshot, self.snapshots.clone());
		let path = create_new_snapshot(object_server, new_snapshot_object)
			.await
			.expect("failed to register backup snapshot");
		self.snapshots.write().await.insert(path);
	}

	async fn delete(
		&self,
		#[zbus(header)] hdr: MessageHeader<'_>,
		#[zbus(object_server)] object_server: &ObjectServer,
	) {
		let btrfs = MountedBtrfs::new().await.expect("failed to mount btrfs");
		btrfs
			.delete_snapshot(&self.metadata)
			.await
			.expect("failed to delete snapshot");
		let metadata_path = btrfs
			.path()
			.join("@snapshots/pop-snapshots")
			.join(self.metadata.uuid.to_string())
			.with_extension("snapshot.json");
		tokio::fs::remove_file(&metadata_path)
			.await
			.expect("failed to remove snapshot metadata");
		let path = OwnedObjectPath::from(
			hdr.path()
				.expect("failed to get own path")
				.expect("invalid object path")
				.to_owned(),
		);
		object_server
			.remove::<Self, _>(&path)
			.await
			.expect("failed to remove object");
		self.snapshots.write().await.remove(&path);
	}
}
