// SPDX-License-Identifier: MPL-2.0

use zbus::{
	dbus_proxy, fdo,
	zvariant::{Optional, OwnedObjectPath},
};

#[dbus_proxy(
	interface = "com.system76.PopSnapshot",
	default_service = "com.system76.PopSnapshot",
	default_path = "/com/system76/PopSnapshot"
)]
pub trait PopSnapshot {
	/// The list of snapshots that are currently registered with the daemon.
	#[dbus_proxy(property)]
	fn snapshots(&self) -> fdo::Result<Vec<OwnedObjectPath>>;

	/// Finds the snapshot with the given UUID.
	fn find_snapshot(&self, uuid: &str) -> fdo::Result<Optional<OwnedObjectPath>>;

	/// Takes a snapshot of the current system state.
	fn create_snapshot(
		&self,
		name: Optional<String>,
		description: Optional<String>,
		subvolumes: Optional<Vec<String>>,
	) -> fdo::Result<OwnedObjectPath>;

	/// Emits a signal when a snapshot is created.
	#[dbus_proxy(signal)]
	fn snapshot_created(&self, uuid: &str) -> fdo::Result<()>;

	/// Emits a signal when a snapshot is deleted.
	#[dbus_proxy(signal)]
	fn snapshot_deleted(&self, uuid: &str) -> fdo::Result<()>;

	/// Emits a signal when a snapshot is restored.
	/// This signal means a reboot is likely imminent.
	#[dbus_proxy(signal)]
	fn snapshot_restored(&self, uuid: &str, backup_uuid: &str) -> fdo::Result<()>;
}
