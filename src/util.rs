// SPDX-License-Identifier: MPL-2.0

use anyhow::{Context, Result};
use libbtrfsutil::{SubvolumeIterator, SubvolumeIteratorFlags};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Finds the btrfs partition that contains the root subvolume.
///
/// This works by scanning /proc/mounts for a mount that has the
/// `subvol=/@root` option.
pub async fn find_root_device() -> Result<PathBuf> {
	let mounts = fs::read_to_string("/proc/mounts")
		.await
		.context("failed to read /proc/mounts")?;
	mounts
		.lines()
		.find(|line| line.contains("subvol=/@root"))
		.and_then(|line| line.split_whitespace().next())
		.map(PathBuf::from)
		.context("failed to find @root")
}

pub fn list_subvolumes_eligible_for_snapshotting(root_path: &Path) -> Result<Vec<String>> {
	let mut subvolumes = Vec::new();
	let info =
		libbtrfsutil::subvolume_info(root_path, None).context("failed to get subvolume info")?;
	let iter = SubvolumeIterator::new(root_path, info.parent_id(), SubvolumeIteratorFlags::empty())
		.context("failed to iterate root subvolumes")?;
	let home_path = PathBuf::from("@home");
	let snapshots_path = PathBuf::from("@snapshots");
	for subvolume in iter {
		let (path, id) = subvolume.context("failed to get subvolume")?;
		debug!("Found subvolume '{}' (id {id})", path.display());
		if path.starts_with(&home_path) || path.starts_with(&snapshots_path) {
			debug!(
				"Skipping subvolume '{}', it is not eligible for snapshotting",
				path.display()
			);
			continue;
		}
		subvolumes.push(path.display().to_string());
	}
	Ok(subvolumes)
}
