// SPDX-License-Identifier: MPL-2.0

pub mod snapshot;

use self::snapshot::SnapshotObject;
use crate::{create_new_snapshot, snapshot::MountedBtrfs, util::ToFdoError};
use anyhow::Context;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;
use zbus::{
	dbus_interface, fdo,
	zvariant::{Optional, OwnedObjectPath},
	ObjectServer, SignalContext,
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

#[dbus_interface(name = "com.system76.PopSnapshot")]
impl SnapshotService {
	#[dbus_interface(property)]
	async fn snapshots(&self) -> Vec<OwnedObjectPath> {
		self.snapshots.read().await.iter().cloned().collect()
	}

	async fn create_snapshot(
		&mut self,
		name: Optional<String>,
		description: Optional<String>,
		#[zbus(signal_context)] ctxt: SignalContext<'_>,
		#[zbus(object_server)] object_server: &ObjectServer,
	) -> fdo::Result<OwnedObjectPath> {
		let btrfs = MountedBtrfs::new()
			.await
			.context("failed to mount btrfs")
			.to_fdo_err()?;
		let snapshot = btrfs
			.create_snapshot(Option::from(name), Option::from(description))
			.await
			.context("failed to create snapshot")
			.to_fdo_err()?;
		let snapshot_uuid = snapshot.uuid.to_string();
		let snapshot_object = SnapshotObject::new(snapshot, self.snapshots.clone());
		let path = create_new_snapshot(object_server, snapshot_object)
			.await
			.with_context(|| format!("failed to register snapshot '{snapshot_uuid}'"))
			.to_fdo_err()?;
		self.snapshots.write().await.insert(path.clone());
		Self::snapshot_created(&ctxt, &snapshot_uuid)
			.await
			.context("failed to emit SnapshotCreated signal")
			.to_fdo_err()?;
		Ok(path)
	}

	#[dbus_interface(signal)]
	async fn snapshot_created(signal_ctxt: &SignalContext<'_>, uuid: &str) -> zbus::Result<()>;

	#[dbus_interface(signal)]
	async fn snapshot_deleted(signal_ctxt: &SignalContext<'_>, uuid: &str) -> zbus::Result<()>;

	#[dbus_interface(signal)]
	async fn snapshot_restored(
		signal_ctxt: &SignalContext<'_>,
		uuid: &str,
		backup_uuid: &str,
	) -> zbus::Result<()>;
}
