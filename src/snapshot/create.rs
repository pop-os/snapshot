// SPDX-License-Identifier: MPL-2.0

use super::MountedBtrfs;
use crate::{service::snapshot::SnapshotObject, util::list_subvolumes_eligible_for_snapshotting};
use anyhow::{Context, Result};
use libbtrfsutil::CreateSnapshotFlags;
use time::OffsetDateTime;

impl MountedBtrfs {
	pub async fn create_snapshot(&self) -> Result<SnapshotObject> {
		let subvolumes_to_snapshot = {
			let path = self.path().to_path_buf();
			tokio::task::spawn_blocking(move || list_subvolumes_eligible_for_snapshotting(&path))
				.await?
				.context("failed to get eligible subvolumes to snapshot")?
		};
		let epoch = OffsetDateTime::now_utc();
		info!(
			"Creating snapshot '{epoch}' with {} subvolumes",
			subvolumes_to_snapshot.len()
		);
		let snapshot_dir = self
			.path()
			.join("@snapshots/pop-snapshots")
			.join(epoch.unix_timestamp().to_string());
		if !snapshot_dir.is_dir() {
			std::fs::create_dir_all(&snapshot_dir).context("failed to create snapshot dir")?;
		}
		let mut subvolumes = Vec::with_capacity(subvolumes_to_snapshot.len());
		for path in subvolumes_to_snapshot {
			let subvolume_name = match path.file_name() {
				Some(name) => name.to_string_lossy().to_string(),
				None => continue,
			};
			subvolumes.push(subvolume_name.clone());
			info!("Snapshotting {}", path.display());
			let source = self.path().join(path);
			let destination = snapshot_dir.join(&subvolume_name);
			tokio::task::spawn_blocking(move || {
				libbtrfsutil::create_snapshot(
					&source,
					&destination,
					CreateSnapshotFlags::READ_ONLY,
					None,
				)
			})
			.await?
			.with_context(|| format!("failed to snapshot subvolume '{}'", subvolume_name))?;
		}
		Ok(SnapshotObject::new(epoch, snapshot_dir, subvolumes))
	}
}
