// SPDX-License-Identifier: MPL-2.0
use anyhow::{anyhow, Context, Result};
use libbtrfsutil::{CreateSnapshotFlags, SubvolumeIterator, SubvolumeIteratorFlags};
use std::{
	path::{Path, PathBuf},
	time::{Duration, SystemTime},
};
use sys_mount::{FilesystemType, Mount, UnmountDrop, UnmountFlags};
use tempfile::TempDir;

#[derive(Debug, Clone)]
pub struct Snapshot {
	capture_time: SystemTime,
	path: PathBuf,
	subvolumes: Vec<String>,
}

pub struct MountedBtrfs {
	_mount: UnmountDrop<Mount>,
	tempdir: TempDir,
}

impl MountedBtrfs {
	pub fn path(&self) -> &Path {
		self.tempdir.path()
	}

	pub fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
		let mut snapshots = Vec::new();
		let snapshot_dir = self.path().join("@snapshots").join("pop-snapshots");
		if !snapshot_dir.exists() {
			return Ok(Vec::new());
		}
		for entry in snapshot_dir
			.read_dir()
			.with_context(|| format!("failed to read directory {}", snapshot_dir.display()))?
		{
			let entry = entry
				.with_context(|| format!("failed to read entry in {}", snapshot_dir.display()))?;
			let path = entry.path();
			if !path.exists() {
				continue;
			}
			let capture_time = match path
				.file_name()
				.and_then(|file_name_os| file_name_os.to_str())
				.and_then(|file_name| file_name.parse::<u64>().ok())
				.map(|seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
			{
				Some(capture_time) => capture_time,
				None => continue,
			};
			let mut subvolumes = Vec::new();
			for entry in path
				.read_dir()
				.with_context(|| format!("failed to read directory {}", path.display()))?
			{
				let entry =
					entry.with_context(|| format!("failed to read entry in {}", path.display()))?;
				let path = entry.path();
				if !path.exists() {
					continue;
				}
				let name = match path
					.file_name()
					.and_then(|file_name_os| file_name_os.to_str())
				{
					Some(name) => name,
					None => continue,
				};
				subvolumes.push(name.to_string());
			}
			snapshots.push(Snapshot {
				capture_time,
				path,
				subvolumes,
			});
		}
		Ok(snapshots)
	}

	pub fn make_snapshot(&self) -> Result<()> {
		let subvolumes_to_snapshot =
			get_non_home_subvolumes().context("failed to list subvolumes")?;
		println!("{} subvolumes to snapshot", subvolumes_to_snapshot.len());
		let epoch = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.context("failed to get current time")?
			.as_secs();
		let snapshot_dir = self
			.path()
			.join("@snapshots")
			.join("pop-snapshots")
			.join(epoch.to_string());
		for path in subvolumes_to_snapshot {
			if !snapshot_dir.is_dir() {
				std::fs::create_dir_all(&snapshot_dir).context("failed to create snapshot dir")?;
			}
			let snapshot_name = format!("pop-snapshot_{epoch}_{}", path.display());
			println!("creating snapshot {snapshot_name}");
			libbtrfsutil::create_snapshot(
				self.path(),
				&snapshot_dir.join(snapshot_name),
				CreateSnapshotFlags::READ_ONLY,
				None,
			)
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
