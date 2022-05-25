// SPDX-License-Identifier: MPL-2.0

use super::MountedBtrfs;
use crate::util::find_root_device;
use anyhow::{Context, Result};
use sys_mount::{FilesystemType, Mount, UnmountFlags};

impl MountedBtrfs {
	/// Mounts the base subvolume of the root btrfs partition in
	/// a temporary directory.
	pub async fn new() -> Result<Self> {
		let tempdir = tempfile::tempdir().context("failed to create tempdir")?;
		let root_device_path = find_root_device()
			.await
			.context("failed to find root device")?;
		let tempdir_path = tempdir.path().to_path_buf();

		let mount = tokio::task::spawn_blocking(move || {
			Mount::builder()
				.fstype(FilesystemType::Manual("btrfs"))
				.data("subvol=/")
				.mount_autodrop(root_device_path, tempdir_path, UnmountFlags::DETACH)
		})
		.await?
		.context("failed to mount root subvolume")?;

		Ok(MountedBtrfs {
			_mount: mount,
			tempdir,
		})
	}
}
