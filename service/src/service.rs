// SPDX-License-Identifier: MPL-2.0

pub mod snapshot;

use self::snapshot::SnapshotObject;
use crate::{config::Config, create_new_snapshot, snapshot::MountedBtrfs, util::ToFdoError};
use anyhow::{anyhow, Context};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;
use zbus::{
	dbus_interface, fdo,
	zvariant::{Optional, OwnedObjectPath},
	ObjectServer, SignalContext,
};

pub struct SnapshotService {
	pub(crate) snapshots: Arc<RwLock<HashMap<Uuid, OwnedObjectPath>>>,
	pub(crate) action_lock: Arc<Mutex<()>>,
	config: Arc<RwLock<Config>>,
}

impl SnapshotService {
	pub fn new(config: Arc<RwLock<Config>>) -> Self {
		Self {
			snapshots: Arc::default(),
			action_lock: Arc::default(),
			config,
		}
	}
}

#[dbus_interface(name = "com.system76.PopSnapshot")]
impl SnapshotService {
	#[dbus_interface(property)]
	async fn snapshots(&self) -> Vec<OwnedObjectPath> {
		self.snapshots.read().await.values().cloned().collect()
	}

	async fn create_snapshot(
		&mut self,
		name: Optional<String>,
		description: Optional<String>,
		subvolumes: Optional<Vec<String>>,
		#[zbus(signal_context)] ctxt: SignalContext<'_>,
		#[zbus(object_server)] object_server: &ObjectServer,
	) -> fdo::Result<OwnedObjectPath> {
		let _lock = match self.action_lock.try_lock() {
			Ok(lock) => lock,
			Err(_) => return Err(anyhow!("pop-snapshot is busy")).to_fdo_err(),
		};
		let btrfs = MountedBtrfs::new()
			.await
			.context("failed to mount btrfs")
			.to_fdo_err()?;
		let snapshot = btrfs
			.create_snapshot(name, description, subvolumes, self.config.clone())
			.await
			.context("failed to create snapshot")
			.to_fdo_err()?;
		let snapshot_uuid = snapshot.uuid;
		let snapshot_object = SnapshotObject::new(
			snapshot,
			self.snapshots.clone(),
			self.action_lock.clone(),
			self.config.clone(),
		);
		let path = create_new_snapshot(object_server, snapshot_object)
			.await
			.with_context(|| format!("failed to register snapshot '{snapshot_uuid}'"))
			.to_fdo_err()?;
		self.snapshots
			.write()
			.await
			.insert(snapshot_uuid, path.clone());
		Self::snapshot_created(&ctxt, &snapshot_uuid.to_string())
			.await
			.context("failed to emit SnapshotCreated signal")
			.to_fdo_err()?;
		Ok(path)
	}

	async fn find_snapshot(&self, uuid: &str) -> fdo::Result<Optional<OwnedObjectPath>> {
		let snapshots = self.snapshots.read().await;
		let uuid = Uuid::parse_str(uuid)
			.with_context(|| format!("failed to parse UUID '{uuid}'", uuid = uuid))
			.to_fdo_err()?;
		let snapshot = snapshots
			.iter()
			.find(|(k, _)| **k == uuid)
			.map(|(_, v)| v.clone());
		Ok(snapshot.into())
	}

	async fn reload_config(&self) -> fdo::Result<()> {
		info!("ReloadConfig called, reloading config");
		let _lock = self.action_lock.lock().await;
		crate::reload_config(self.config.clone())
			.await
			.context("failed to reload config")
			.to_fdo_err()
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
