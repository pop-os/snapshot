// SPDX-License-Identifier: MPL-2.0

mod create;
mod list;
mod mount;
mod restore;

use anyhow::{anyhow, Context, Result};
use libbtrfsutil::{CreateSnapshotFlags, SubvolumeIterator, SubvolumeIteratorFlags};
use std::{
	path::{Path, PathBuf},
	time::{Duration, SystemTime},
};
use sys_mount::{FilesystemType, Mount, UnmountDrop, UnmountFlags};
use tempfile::TempDir;
use tokio::fs;

pub struct MountedBtrfs {
	_mount: UnmountDrop<Mount>,
	tempdir: TempDir,
}

impl MountedBtrfs {
	pub fn path(&self) -> &Path {
		self.tempdir.path()
	}

	pub async fn restore_snapshot(&self, snapshot: u64) -> Result<()> {
		let snapshot_dir = self
			.path()
			.join("@snapshots")
			.join("pop-snapshots")
			.join(snapshot.to_string());
		if !snapshot_dir.exists() {
			return Err(anyhow!("snapshot {} does not exist", snapshot));
		}
		let epoch = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.context("failed to get current time")?
			.as_secs();
		let current_snapshot_dir = self
			.path()
			.join("@snapshots")
			.join("pop-snapshots")
			.join(epoch.to_string());
		if !current_snapshot_dir.exists() {
			fs::create_dir_all(&current_snapshot_dir)
				.await
				.with_context(|| {
					format!(
						"failed to create directory {}",
						current_snapshot_dir.display()
					)
				})?;
		}
		let mut dir = fs::read_dir(&snapshot_dir)
			.await
			.with_context(|| format!("failed to read directory {}", snapshot_dir.display()))?;
		while let Some(entry) = dir
			.next_entry()
			.await
			.context("failed to read directory entry")?
		{
			let path = entry.path();
			let name = match path
				.file_name()
				.and_then(|file_name_os| file_name_os.to_str())
			{
				Some(name) => name,
				None => continue,
			};
			let base_subvolume_path = self.path().join(name);
			if !base_subvolume_path.exists() {
				continue;
			}
			let new_snapshot = current_snapshot_dir.join(name);
			fs::rename(&path, &new_snapshot).await.with_context(|| {
				format!(
					"failed to rename {} to {}",
					path.display(),
					new_snapshot.display()
				)
			})?;
			libbtrfsutil::create_snapshot(
				&path,
				&base_subvolume_path,
				CreateSnapshotFlags::empty(),
				None,
			)
			.with_context(|| format!("failed to snapshot subvolume '{}'", path.display()))?;
		}

		Ok(())
	}
}

pub fn get_non_home_subvolumes() -> Result<Vec<PathBuf>> {
	let info = libbtrfsutil::subvolume_info("/", None).context("failed to get subvolume info")?;
	dbg!(&info);
	let iter = SubvolumeIterator::new("/", info.parent_id(), SubvolumeIteratorFlags::empty())
		.context("failed to iterate root subvolumes")?;
	let home_path = PathBuf::from("@home");
	let snapshots_path = PathBuf::from("@snapshots");
	let mut subvolumes = Vec::new();
	for subvolume in iter {
		let (path, id) = subvolume.context("failed to get subvolume")?;
		println!("found {} (id {id})", path.display());
		if path == home_path || path == snapshots_path {
			continue;
		}
		subvolumes.push(path);
	}
	Ok(subvolumes)
}
