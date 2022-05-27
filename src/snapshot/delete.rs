// SPDX-License-Identifier: MPL-2.0

use super::{metadata::SnapshotMetadata, MountedBtrfs};
use anyhow::{anyhow, Context, Result};
use libbtrfsutil::DeleteSubvolumeFlags;
use tokio::fs;

impl MountedBtrfs {
	pub async fn delete_snapshot(&self, snapshot: &SnapshotMetadata) -> Result<()> {
		let snapshot_dir = self
			.path()
			.join("@snapshots/pop-snapshots")
			.join(snapshot.uuid.to_string());
		if !snapshot_dir.exists() {
			return Err(anyhow!("snapshot {} does not exist", snapshot.uuid));
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
			info!("deleting subvolume at {}", path.display());
			tokio::task::spawn_blocking(move || {
				libbtrfsutil::delete_subvolume(&path, DeleteSubvolumeFlags::empty())
			})
			.await?
			.context("failed to delete subvolume")?;
		}
		fs::remove_dir_all(&snapshot_dir)
			.await
			.with_context(|| format!("failed to delete directory {}", snapshot_dir.display()))
	}
}
