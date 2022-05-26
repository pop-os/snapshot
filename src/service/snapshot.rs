// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use zbus::dbus_interface;

use crate::snapshot::MountedBtrfs;

pub struct SnapshotObject {
	creation_time: OffsetDateTime,
	path: PathBuf,
	subvolumes: Vec<String>,
}

impl SnapshotObject {
	pub(crate) fn new(
		creation_time: OffsetDateTime,
		path: PathBuf,
		subvolumes: Vec<String>,
	) -> Self {
		Self {
			creation_time,
			path,
			subvolumes,
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
}
