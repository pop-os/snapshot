// SPDX-License-Identifier: MPL-2.0
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use sys_mount::{FilesystemType, Mount, UnmountDrop, UnmountFlags};
use tempfile::TempDir;

pub struct MountedSnapshots {
	mount: UnmountDrop<Mount>,
	tempdir: TempDir,
}

impl MountedSnapshots {
	pub fn path(&self) -> &Path {
		self.tempdir.path()
	}
}

pub fn find_root_device() -> Result<PathBuf> {
	let mounts = std::fs::read_to_string("/proc/mounts").context("failed to read /proc/mounts")?;
	mounts
		.lines()
		.find(|line| line.contains("subvol=@root"))
		.and_then(|line| line.split_whitespace().next())
		.map(PathBuf::from)
		.context("failed to find @root")
}

pub fn mount_snapshot_folder() -> Result<MountedSnapshots> {
	let tempdir = tempfile::tempdir().context("failed to create tempdir")?;
	let mount = Mount::builder()
		.fstype(FilesystemType::Manual("btrfs"))
		.data("subvol=@snapshots")
		.mount_autodrop(
			find_root_device().context("failed to find root device")?,
			tempdir.path(),
			UnmountFlags::DETACH,
		)
		.with_context(|| {
			format!(
				"failed to mount snapshot folder to {}",
				tempdir.path().display()
			)
		})?;
	Ok(MountedSnapshots { mount, tempdir })
}
