// SPDX-License-Identifier: MPL-2.0

use super::SnapshotService;
use crate::{
	create_new_snapshot,
	snapshot::{metadata::SnapshotMetadata, MountedBtrfs},
	util::ToFdoError,
};
use anyhow::{Context, Result};
use std::{collections::HashSet, sync::Arc};
use time::format_description::well_known::Rfc3339;
use tokio::sync::RwLock;
use zbus::{
	dbus_interface, fdo, zvariant::OwnedObjectPath, Connection, MessageHeader, ObjectServer,
	SignalContext,
};

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

	async fn get_base_service(&self, conn: &Connection) -> zbus::Result<SignalContext<'_>> {
		let path = OwnedObjectPath::try_from("/com/system76/SnapshotDaemon")?;
		SignalContext::new(conn, path)
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
	async fn set_name(&mut self, value: &str) -> fdo::Result<()> {
		self.metadata.name = if value.trim().is_empty() {
			None
		} else {
			Some(value.to_owned())
		};
		self.update_metadata_file()
			.await
			.context("failed to update metadata file")
			.to_fdo_err()?;
		Ok(())
	}

	#[dbus_interface(property)]
	async fn description(&self) -> String {
		self.metadata.description.clone().unwrap_or_default()
	}

	#[dbus_interface(property)]
	async fn set_description(&mut self, value: &str) -> fdo::Result<()> {
		self.metadata.description = if value.trim().is_empty() {
			None
		} else {
			Some(value.to_owned())
		};
		self.update_metadata_file()
			.await
			.context("failed to update metadata file")
			.to_fdo_err()?;
		Ok(())
	}

	#[dbus_interface(property)]
	async fn subvolumes(&self) -> Vec<String> {
		self.metadata.subvolumes.clone()
	}

	#[dbus_interface(property)]
	async fn uuid(&self) -> String {
		self.metadata.uuid.to_string()
	}

	async fn restore(
		&self,
		#[zbus(connection)] connection: &Connection,
		#[zbus(object_server)] object_server: &ObjectServer,
	) -> fdo::Result<()> {
		let btrfs = MountedBtrfs::new()
			.await
			.context("failed to mount btrfs")
			.to_fdo_err()?;
		let new_snapshot = btrfs
			.restore_snapshot(&self.metadata)
			.await
			.context("failed to restore snapshot")
			.to_fdo_err()?;
		let new_snapshot_uuid = new_snapshot.uuid.to_string();
		let new_snapshot_object = SnapshotObject::new(new_snapshot, self.snapshots.clone());
		let path = create_new_snapshot(object_server, new_snapshot_object)
			.await
			.context("failed to register backup snapshot")
			.to_fdo_err()?;
		self.snapshots.write().await.insert(path);
		let base_service = self
			.get_base_service(connection)
			.await
			.context("failed to get base service signal context")
			.to_fdo_err()?;
		SnapshotService::snapshot_restored(
			&base_service,
			&self.metadata.uuid.to_string(),
			&new_snapshot_uuid,
		)
		.await
		.context("failed to emit SnapshotRestored signal")
		.to_fdo_err()?;
		SnapshotService::snapshot_created(&base_service, &new_snapshot_uuid)
			.await
			.context("failed to emit SnapshotCreated signal")
			.to_fdo_err()?;
		Ok(())
	}

	async fn delete(
		&self,
		#[zbus(connection)] connection: &Connection,
		#[zbus(header)] hdr: MessageHeader<'_>,
		#[zbus(object_server)] object_server: &ObjectServer,
	) -> fdo::Result<()> {
		let btrfs = MountedBtrfs::new()
			.await
			.context("failed to mount btrfs")
			.to_fdo_err()?;
		btrfs
			.delete_snapshot(&self.metadata)
			.await
			.context("failed to delete snapshot")
			.to_fdo_err()?;
		let metadata_path = btrfs
			.path()
			.join("@snapshots/pop-snapshots")
			.join(self.metadata.uuid.to_string())
			.with_extension("snapshot.json");
		tokio::fs::remove_file(&metadata_path)
			.await
			.context("failed to remove snapshot metadata")
			.to_fdo_err()?;
		let path = OwnedObjectPath::from(
			hdr.path()
				.context("failed to get own path")
				.to_fdo_err()?
				.context("invalid object path")
				.to_fdo_err()?
				.to_owned(),
		);
		object_server
			.remove::<Self, _>(&path)
			.await
			.context("failed to remove object")
			.to_fdo_err()?;
		self.snapshots.write().await.remove(&path);
		let base_service = self
			.get_base_service(connection)
			.await
			.context("failed to get base service signal context")
			.to_fdo_err()?;
		SnapshotService::snapshot_deleted(&base_service, &self.metadata.uuid.to_string())
			.await
			.context("failed to emit SnapshotDeleted signal")
			.to_fdo_err()?;
		Ok(())
	}
}