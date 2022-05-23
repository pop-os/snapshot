// SPDX-License-Identifier: MPL-2.0
use anyhow::{anyhow, Context, Result};
use libbtrfsutil::{CreateSnapshotFlags, SubvolumeInfo, SubvolumeIterator, SubvolumeIteratorFlags};
use std::{
	path::{Path, PathBuf},
	time::SystemTime,
};
use sys_mount::{FilesystemType, Mount, UnmountDrop, UnmountFlags};
use tempfile::TempDir;

pub struct MountedBtrfs {
	_mount: UnmountDrop<Mount>,
	tempdir: TempDir,
}

impl MountedBtrfs {
	pub fn path(&self) -> &Path {
		self.tempdir.path()
	}

	pub fn make_snapshot(&self) -> Result<()> {
		let subvolumes_to_snapshot =
			get_non_home_subvolumes().context("failed to list subvolumes")?;
		println!("{} subvolumes to snapshot", subvolumes_to_snapshot.len());
		let epoch = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.context("failed to get current time")?
			.as_secs();
		let snapshots_dir = self.tempdir.path().join("@snapshots");
		for path in subvolumes_to_snapshot {
			let snapshot_name = format!("pop-snapshot_{epoch}_{}", path.display());
			println!("creating snapshot {snapshot_name}");
			libbtrfsutil::create_snapshot(
				self.path(),
				&snapshots_dir.join(snapshot_name),
				CreateSnapshotFlags::READ_ONLY,
				None,
			)
			.map_err(|err| anyhow!("btrfs error {err:?}"))
			.with_context(|| format!("failed to snapshot subvolume '{}'", path.display()))?;
		}
		Ok(())
	}
}

pub fn find_root_device() -> Result<PathBuf> {
	let mounts = std::fs::read_to_string("/proc/mounts").context("failed to read /proc/mounts")?;
	mounts
		.lines()
		.find(|line| line.contains("subvol=/@root"))
		.and_then(|line| line.split_whitespace().next())
		.map(PathBuf::from)
		.context("failed to find @root")
}

pub fn mount_snapshot_folder() -> Result<MountedBtrfs> {
	let tempdir = tempfile::tempdir().context("failed to create tempdir")?;
	let mount = Mount::builder()
		.fstype(FilesystemType::Manual("btrfs"))
		.data("subvol=/")
		.mount_autodrop(
			find_root_device().context("failed to find root device")?,
			tempdir.path(),
			UnmountFlags::DETACH,
		)
		.with_context(|| format!("failed to mount btrfs root to {}", tempdir.path().display()))?;
	Ok(MountedBtrfs {
		_mount: mount,
		tempdir,
	})
}

pub fn get_non_home_subvolumes() -> Result<Vec<PathBuf>> {
	let info = libbtrfsutil::subvolume_info("/", None)
		.map_err(|err| anyhow!("btrfs error {err:?}"))
		.context("failed to get subvolume info")?;
	dbg!(&info);
	let iter = SubvolumeIterator::new("/", info.parent_id(), SubvolumeIteratorFlags::empty())
		.map_err(|err| anyhow!("btrfs error {err:?}"))
		.context("failed to iterate root subvolumes")?;
	let home_path = PathBuf::from("@home");
	let snapshots_path = PathBuf::from("@snapshots");
	let mut subvolumes = Vec::new();
	for subvolume in iter {
		let (path, id) = subvolume
			.map_err(|err| anyhow!("btrfs error {err:?}"))
			.context("failed to get subvolume")?;
		println!("found {} (id {id})", path.display());
		if path == home_path || path == snapshots_path {
			continue;
		}
		subvolumes.push(path);
	}
	Ok(subvolumes)
}
