// SPDX-License-Identifier: MPL-2.0

use zbus::dbus_interface;

pub struct SnapshotService;

#[dbus_interface(name = "com.system76.SnapshotDaemon")]
impl SnapshotService {
	async fn list_snapshots(&self) {
		todo!()
	}

	async fn take_snapshot(&self) {
		todo!()
	}

	async fn restore_snapshot(&self, snapshot: u64) {
		todo!()
	}
}
