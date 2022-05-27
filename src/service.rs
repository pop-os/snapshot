// SPDX-License-Identifier: MPL-2.0

pub mod snapshot;

use self::snapshot::SnapshotObject;
use crate::{create_new_snapshot, snapshot::MountedBtrfs};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;
use zbus::{
	dbus_interface,
	zvariant::{Optional, OwnedObjectPath},
	ObjectServer,
};

pub struct SnapshotService {
	pub(crate) snapshots: Arc<RwLock<HashSet<OwnedObjectPath>>>,
}

impl SnapshotService {
	pub fn new() -> Self {
		Self {
			snapshots: Arc::default(),
		}
	}
}

#[dbus_interface(name = "com.system76.SnapshotDaemon")]
impl SnapshotService {
	#[dbus_interface(property)]
	async fn snapshots(&self) -> Vec<OwnedObjectPath> {
		self.snapshots.read().await.iter().cloned().collect()
	}

	async fn create_snapshot(
		&mut self,
		name: Optional<String>,
		description: Optional<String>,
		#[zbus(object_server)] object_server: &ObjectServer,
	) -> OwnedObjectPath {
		let btrfs = MountedBtrfs::new().await.expect("failed to mount btrfs");
		let snapshot = btrfs
			.create_snapshot(Option::from(name), Option::from(description))
			.await
			.expect("failed to create snapshot");
		let snapshot_object = SnapshotObject::new(snapshot, self.snapshots.clone());
		let path = create_new_snapshot(object_server, snapshot_object)
			.await
			.expect("failed to register snapshot");
		self.snapshots.write().await.insert(path.clone());
		path
	}
}
