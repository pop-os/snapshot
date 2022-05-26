// SPDX-License-Identifier: MPL-2.0

use std::{collections::HashSet, path::PathBuf, sync::Arc};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::sync::RwLock;
use zbus::{dbus_interface, zvariant::OwnedObjectPath, MessageHeader, ObjectServer};

use crate::snapshot::MountedBtrfs;

pub struct SnapshotObject {
	creation_time: OffsetDateTime,
	path: PathBuf,
	subvolumes: Vec<String>,
	snapshots: Arc<RwLock<HashSet<OwnedObjectPath>>>,
}

impl SnapshotObject {
	pub(crate) fn new(
		creation_time: OffsetDateTime,
		path: PathBuf,
		subvolumes: Vec<String>,
		snapshots: Arc<RwLock<HashSet<OwnedObjectPath>>>,
	) -> Self {
		Self {
			creation_time,
			path,
			subvolumes,
			snapshots,
		}
	}
}

#[dbus_interface(name = "com.system76.SnapshotDaemon.Snapshot")]
impl SnapshotObject {
	#[dbus_interface(property)]
	async fn creation_time(&self) -> String {
		self.creation_time
			.format(&Rfc3339)
			.expect("failed to format time as RFC 3399")
	}

	#[dbus_interface(property)]
	async fn path(&self) -> String {
		self.path
			.as_os_str()
			.to_str()
			.expect("invalid path")
			.to_string()
	}

	#[dbus_interface(property)]
	async fn subvolumes(&self) -> Vec<String> {
		self.subvolumes.clone()
	}

	async fn restore(&self) {
		let btrfs = MountedBtrfs::new().await.expect("failed to mount btrfs");
		btrfs
			.restore_snapshot(self.creation_time)
			.await
			.expect("failed to restore snapshot");
	}

	async fn delete(
		&self,
		#[zbus(header)] hdr: MessageHeader<'_>,
		#[zbus(object_server)] object_server: &ObjectServer,
	) {
		let btrfs = MountedBtrfs::new().await.expect("failed to mount btrfs");
		btrfs
			.delete_snapshot(self.creation_time)
			.await
			.expect("failed to delete snapshot");
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
