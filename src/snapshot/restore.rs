// SPDX-License-Identifier: MPL-2.0

use super::MountedBtrfs;
use anyhow::{anyhow, Context, Result};
use libbtrfsutil::CreateSnapshotFlags;
use time::OffsetDateTime;
use tokio::fs;

impl MountedBtrfs {
	pub async fn restore_snapshot(&self, snapshot: OffsetDateTime) -> Result<()> {
		let restore_snapshot_dir = self
			.path()
			.join("@snapshots/pop-snapshots")
			.join(snapshot.unix_timestamp().to_string());
		if !restore_snapshot_dir.exists() {
			return Err(anyhow!("snapshot {} does not exist", snapshot));
		}
		let epoch = OffsetDateTime::now_utc();
		let current_snapshot_dir = self
			.path()
			.join("@snapshots/pop-snapshots")
			.join(epoch.unix_timestamp().to_string());
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
		let mut dir = fs::read_dir(&restore_snapshot_dir).await.with_context(|| {
			format!(
				"failed to read directory {}",
				restore_snapshot_dir.display()
			)
		})?;
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
			info!(
				"{} -> {}",
				base_subvolume_path.display(),
				new_snapshot.display()
			);
			fs::rename(&base_subvolume_path, &new_snapshot)
				.await
				.with_context(|| {
					format!(
						"failed to rename {} to {}",
						path.display(),
						new_snapshot.display()
					)
				})?;
			info!(
				"snapshotting {} to {}",
				path.display(),
				base_subvolume_path.display()
			);
			let source = path.clone();
			tokio::task::spawn_blocking(move || {
				libbtrfsutil::create_snapshot(
					&source,
					&base_subvolume_path,
					CreateSnapshotFlags::empty(),
					None,
				)
			})
			.await?
			.with_context(|| format!("failed to snapshot subvolume '{}'", path.display()))?;
		}

		Ok(())
	}
}
