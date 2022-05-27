// SPDX-License-Identifier: MPL-2.0

use super::{metadata::SnapshotMetadata, MountedBtrfs};
use anyhow::{anyhow, Context, Result};
use libbtrfsutil::CreateSnapshotFlags;
use tokio::fs;

impl MountedBtrfs {
	pub async fn restore_snapshot(&self, snapshot: &SnapshotMetadata) -> Result<SnapshotMetadata> {
		let restore_snapshot_dir = self
			.path()
			.join("@snapshots/pop-snapshots")
			.join(snapshot.uuid.to_string());
		if !restore_snapshot_dir.exists() {
			return Err(anyhow!("snapshot {} does not exist", snapshot.uuid));
		}
		let new_snapshot = SnapshotMetadata::now(
			None,
			format!(
				"Automatic snapshot made when restoring snapshot {}",
				snapshot.uuid
			),
			snapshot.subvolumes.clone(),
		);
		let new_snapshot_dir = self
			.path()
			.join("@snapshots/pop-snapshots")
			.join(new_snapshot.uuid.to_string());
		if !new_snapshot_dir.exists() {
			fs::create_dir_all(&new_snapshot_dir)
				.await
				.with_context(|| {
					format!("failed to create directory {}", new_snapshot_dir.display())
				})?;
		}
		for subvolume in &snapshot.subvolumes {
			let restore_target_subvolume_path =
				restore_snapshot_dir.join(subvolume.replace('/', "__"));
			let new_snapshot_subvolume_path = new_snapshot_dir.join(subvolume.replace('/', "__"));
			let subvolume_path = self.path().join(subvolume);
			info!(
				"{} -> {}",
				subvolume_path.display(),
				new_snapshot_subvolume_path.display()
			);
			fs::rename(&subvolume_path, &new_snapshot_subvolume_path)
				.await
				.with_context(|| {
					format!(
						"failed to rename {} to {}",
						subvolume_path.display(),
						new_snapshot_subvolume_path.display()
					)
				})?;
			info!(
				"snapshotting {} to {}",
				restore_target_subvolume_path.display(),
				subvolume_path.display()
			);
			let source = restore_target_subvolume_path.clone();
			tokio::task::spawn_blocking(move || {
				libbtrfsutil::create_snapshot(
					&source,
					&subvolume_path,
					CreateSnapshotFlags::empty(),
					None,
				)
			})
			.await?
			.with_context(|| {
				format!(
					"failed to snapshot subvolume '{}'",
					restore_target_subvolume_path.display()
				)
			})?;
		}

		let new_snapshot_metadata_path = self
			.path()
			.join("@snapshots/pop-snapshots")
			.join(new_snapshot.uuid.to_string())
			.with_extension("snapshot.json");
		info!(
			"writing new snapshot metadata to {}",
			new_snapshot_metadata_path.display()
		);
		fs::write(
			&new_snapshot_metadata_path,
			serde_json::to_string_pretty(&new_snapshot)?,
		)
		.await
		.with_context(|| {
			format!(
				"failed to write snapshot metadata to {}",
				new_snapshot_metadata_path.display()
			)
		})?;

		Ok(new_snapshot)
	}
}
