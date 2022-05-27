// SPDX-License-Identifier: MPL-2.0

use super::{metadata::SnapshotMetadata, MountedBtrfs};
use crate::util::list_subvolumes_eligible_for_snapshotting;
use anyhow::{Context, Result};
use libbtrfsutil::CreateSnapshotFlags;
impl MountedBtrfs {
	pub async fn create_snapshot(
		&self,
		name: impl Into<Option<String>>,
		description: impl Into<Option<String>>,
	) -> Result<SnapshotMetadata> {
		let subvolumes_to_snapshot = {
			let path = self.path().to_path_buf();
			tokio::task::spawn_blocking(move || list_subvolumes_eligible_for_snapshotting(&path))
				.await?
				.context("failed to get eligible subvolumes to snapshot")?
		};
		let num_subvolumes = subvolumes_to_snapshot.len();
		let snapshot = SnapshotMetadata::now(name, description, subvolumes_to_snapshot);
		info!(
			"Creating snapshot '{}' with {num_subvolumes} subvolumes",
			snapshot.uuid
		);
		let snapshot_dir = self
			.path()
			.join("@snapshots/pop-snapshots")
			.join(snapshot.uuid.to_string());
		if !snapshot_dir.is_dir() {
			std::fs::create_dir_all(&snapshot_dir).context("failed to create snapshot dir")?;
		}
		for subvolume in &snapshot.subvolumes {
			info!("Snapshotting {subvolume}");
			let source = self.path().join(subvolume);
			let destination = snapshot_dir.join(&subvolume.replace("/", "__"));
			tokio::task::spawn_blocking(move || {
				libbtrfsutil::create_snapshot(
					&source,
					&destination,
					CreateSnapshotFlags::READ_ONLY,
					None,
				)
			})
			.await?
			.with_context(|| format!("failed to snapshot subvolume '{}'", subvolume))?;
		}

		let snapshot_metadata_path = self
			.path()
			.join("@snapshots/pop-snapshots")
			.join(snapshot.uuid.to_string())
			.with_extension("snapshot.json");
		tokio::fs::write(
			&snapshot_metadata_path,
			serde_json::to_string_pretty(&snapshot)?,
		)
		.await
		.with_context(|| {
			format!(
				"failed to write snapshot metadata to '{}'",
				snapshot_metadata_path.display()
			)
		})?;

		Ok(snapshot)
	}
}
