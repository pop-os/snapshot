// SPDX-License-Identifier: MPL-2.0

use super::MountedBtrfs;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use tokio::fs;

#[derive(Debug, Clone)]
pub struct Snapshot {
	/// The time at which this snapshot was made,
	/// represented as a unix timestamp.
	pub(crate) capture_time: OffsetDateTime,
	/// The path this snapshot currently lives at.
	pub(crate) path: PathBuf,
	/// The subvolumes this snapshot contains.
	pub(crate) subvolumes: Vec<String>,
}

impl MountedBtrfs {
	async fn list_snapshots_base(&self) -> Result<Vec<(PathBuf, OffsetDateTime)>> {
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
			let capture_time = match path_to_date_time(&path) {
				Some(capture_time) => capture_time,
				None => continue,
			};
			debug!("Found snapshot {capture_time} at {}", path.display());
			snapshots.push((path, capture_time));
		}
		Ok(snapshots)
	}

	async fn list_snapshot_subvolumes(&self, capture_time: OffsetDateTime) -> Result<Vec<String>> {
		let mut subvolumes = Vec::new();
		let snapshot_dir = self
			.path()
			.join("@snapshots/pop-snapshots")
			.join(capture_time.unix_timestamp().to_string());
		if !snapshot_dir.exists() {
			return Err(anyhow!("snapshot taken at '{capture_time}' does not exist"));
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
			let subvolume = match path
				.file_name()
				.map(|file_name_os| file_name_os.to_string_lossy().into_owned())
			{
				Some(subvolume) => subvolume,
				None => continue,
			};
			debug!(
				"Found subvolume {subvolume} for snapshot {capture_time} at {}",
				path.display()
			);
			subvolumes.push(subvolume);
		}
		Ok(subvolumes)
	}

	pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
		let mut snapshots = Vec::new();
		let snapshots_taken = self
			.list_snapshots_base()
			.await
			.context("failed to list snapshots")?;
		for (path, snapshot) in snapshots_taken {
			let subvolumes = self
				.list_snapshot_subvolumes(snapshot)
				.await
				.with_context(|| format!("failed to list subvolumes for snapshot '{snapshot}'"))?;
			debug!(
				"Snapshot {snapshot} has {} subvolumes at {}",
				subvolumes.len(),
				path.display()
			);
			snapshots.push(Snapshot {
				capture_time: snapshot,
				path,
				subvolumes,
			});
		}
		Ok(snapshots)
	}
}

pub fn path_to_date_time(path: &Path) -> Option<OffsetDateTime> {
	path.file_name()
		.and_then(|file_name_os| file_name_os.to_str())
		.and_then(|file_name| file_name.parse::<u64>().ok())
		.and_then(|seconds| OffsetDateTime::from_unix_timestamp(seconds as i64).ok())
}
