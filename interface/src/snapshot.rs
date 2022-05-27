// SPDX-License-Identifier: MPL-2.0

use zbus::{dbus_proxy, fdo};

#[dbus_proxy(
	interface = "com.system76.SnapshotDaemon.Snapshot",
	default_service = "com.system76.SnapshotDaemon"
)]
pub trait Snapshot {
	/// The time at which this snapshot was made, in RFC3339 format.
	#[dbus_proxy(property)]
	fn creation_time(&self) -> fdo::Result<String>;

	/// The name of this snapshot, if there is any.
	#[dbus_proxy(property)]
	fn name(&self) -> fdo::Result<String>;

	/// Sets the name of this snapshot.
	/// A blank value will remove the name.
	#[dbus_proxy(property)]
	fn set_name(&self, name: &str) -> fdo::Result<()>;

	/// The description of this snapshot, if there is any.
	#[dbus_proxy(property)]
	fn description(&self) -> fdo::Result<String>;

	/// Sets the description of this snapshot.
	/// A blank value will remove the description.
	#[dbus_proxy(property)]
	fn set_description(&self, description: &str) -> fdo::Result<()>;

	/// A list of subvolumes that have been captured by this snapshot.
	#[dbus_proxy(property)]
	fn subvolumes(&self) -> fdo::Result<Vec<String>>;

	/// The unique identifier of this snapshot.
	#[dbus_proxy(property)]
	fn uuid(&self) -> fdo::Result<String>;

	/// Restores the system to this snapshot,
	/// creating a backup snapshot of the current system state in the process.
	fn restore(&self) -> fdo::Result<()>;

	/// Deletes this snapshot permanently.
	fn delete(&self) -> fdo::Result<()>;
}
