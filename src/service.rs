// SPDX-License-Identifier: MPL-2.0

use crate::snapshot::MountedBtrfs;
use zbus::dbus_interface;

pub struct SnapshotService;

#[dbus_interface(name = "com.system76.SnapshotDaemon")]
impl SnapshotService {
	async fn list_snapshots(&self) {
		todo!()
	}

	async fn take_snapshot(&self) {
		let btrfs = MountedBtrfs::new().expect("failed to mount btrfs");
		btrfs.make_snapshot().expect("failed to take snapshot");
	}

	async fn restore_snapshot(&self, snapshot: u64) {
		let btrfs = MountedBtrfs::new().expect("failed to mount btrfs");
		btrfs
			.restore_snapshot(snapshot)
			.await
			.expect("failed to restore snapshot");
	}
}
