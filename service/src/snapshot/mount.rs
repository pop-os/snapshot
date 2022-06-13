// SPDX-License-Identifier: MPL-2.0

use super::MountedBtrfs;
use crate::util::find_root_device;
use anyhow::{Context, Result};
use libbtrfsutil::CreateSubvolumeFlags;
use sys_mount::{FilesystemType, Mount, UnmountFlags};

impl MountedBtrfs {
	/// Mounts the base subvolume of the root btrfs partition in
	/// a temporary directory.
	pub async fn new() -> Result<Self> {
		let tempdir = tempfile::tempdir().context("failed to create tempdir")?;
		let root_device_path = find_root_device()
			.await
			.context("failed to find root device")?;
		debug!("Found root device path at {}", root_device_path.display());
		let tempdir_path = tempdir.path().to_path_buf();
		let snapshots_path = tempdir_path.join("@snapshots");

		debug!(
			"Mounting {}[subvol=/] at {}",
			root_device_path.display(),
			tempdir_path.display()
		);
		let mount = tokio::task::spawn_blocking(move || {
			Mount::builder()
				.fstype(FilesystemType::Manual("btrfs"))
				.data("subvol=/")
				.mount_autodrop(root_device_path, tempdir_path, UnmountFlags::DETACH)
		})
		.await?
		.context("failed to mount root subvolume")?;

		if !snapshots_path.exists() {
			let pop_snapshots_dir = snapshots_path.join("pop-snapshots");
			tokio::task::spawn_blocking(move || {
				libbtrfsutil::create_subvolume(snapshots_path, CreateSubvolumeFlags::empty(), None)
			})
			.await?
			.context("failed to create snapshot subvolume")?;
			tokio::fs::create_dir(&pop_snapshots_dir)
				.await
				.with_context(|| format!("failed to create {}", pop_snapshots_dir.display()))?;
		}

		Ok(MountedBtrfs {
			_mount: mount,
			tempdir,
		})
	}
}
