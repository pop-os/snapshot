// SPDX-License-Identifier: MPL-2.0

use super::{metadata::SnapshotMetadata, MountedBtrfs};
use anyhow::{anyhow, Context, Result};
use tokio::fs;

impl MountedBtrfs {
	pub async fn list_snapshots(&self) -> Result<Vec<SnapshotMetadata>> {
		let mut snapshots = Vec::new();
		let snapshot_dir = self.path().join("@snapshots/pop-snapshots");
		if !snapshot_dir.exists() {
			return Ok(Vec::new());
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
			if !path.is_file() {
				continue;
			}
			let name = match path
				.file_name()
				.and_then(|file_name_os| file_name_os.to_str())
			{
				Some(name) => name.to_owned(),
				None => {
					return Err(anyhow!(
						"failed to get file name from path {}",
						path.display()
					));
				}
			};
			if !name.ends_with(".snapshot.json") {
				continue;
			}
			let metadata: SnapshotMetadata = serde_json::from_str(
				&fs::read_to_string(&path)
					.await
					.context(format!("failed to read file {}", path.display()))?,
			)
			.with_context(|| format!("failed to parse metadata from file {}", path.display()))?;
			snapshots.push(metadata);
		}
		Ok(snapshots)
	}
}
