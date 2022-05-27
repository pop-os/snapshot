// SPDX-License-Identifier: MPL-2.0

use crate::{create_new_snapshot, snapshot::metadata::SnapshotMetadata};
use std::{collections::HashSet, sync::Arc};
use time::format_description::well_known::Rfc3339;
use tokio::sync::RwLock;
use zbus::{dbus_interface, zvariant::OwnedObjectPath, MessageHeader, ObjectServer};

use crate::snapshot::MountedBtrfs;

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
	async fn description(&self) -> String {
		self.metadata.description.clone().unwrap_or_default()
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
