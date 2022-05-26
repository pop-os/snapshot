// SPDX-License-Identifier: MPL-2.0
pub(crate) mod service;
pub(crate) mod snapshot;
pub(crate) mod util;

#[macro_use]
extern crate tracing;

use crate::{service::snapshot::SnapshotObject, snapshot::list::Snapshot};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use zbus::{zvariant::OwnedObjectPath, ConnectionBuilder, ObjectServer};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

async fn create_new_snapshot(
	object_server: &ObjectServer,
	snapshot_object: SnapshotObject,
) -> Result<OwnedObjectPath> {
	let new_id = COUNTER.fetch_add(1, Ordering::SeqCst);
	let id =
		OwnedObjectPath::try_from(format!("/com/system76/SnapshotDaemon/Snapshot/{}", new_id))?;
	object_server
		.at(&id, snapshot_object)
		.await
		.with_context(|| format!("failed to register snapshot {:?}", id))?;
	Ok(id)
}

#[tokio::main]
async fn main() -> Result<()> {
	// Set up the tracing logger.
	tracing_subscriber::registry()
		.with(fmt::layer())
		.with(
			EnvFilter::builder()
				.with_default_directive(LevelFilter::DEBUG.into())
				.from_env_lossy(),
		)
		.init();

	let mut service = service::SnapshotService::new();
	let connection = ConnectionBuilder::system()
		.context("failed to get system dbus connection")?
		.name("com.system76.SnapshotDaemon")?
		//.serve_at("/com/system76/SnapshotDaemon", service)?
		.build()
		.await
		.context("failed to build connection")?;

	{
		let btrfs = snapshot::MountedBtrfs::new()
			.await
			.context("failed to mount btrfs to list snapshots")?;
		let snapshots = btrfs
			.list_snapshots()
			.await
			.context("failed to list snapshots")?;
		service.snapshots.reserve(snapshots.len());
		for Snapshot {
			capture_time,
			path,
			subvolumes,
		} in snapshots
		{
			let snapshot_object = SnapshotObject::new(capture_time, path, subvolumes);
			let id = create_new_snapshot(&*connection.object_server(), snapshot_object)
				.await
				.context("failed to create new snapshot object")?;

			debug!("Created new snapshot object: {:?}", id);
			service.snapshots.push(id);
		}
	}
	connection
		.object_server()
		.at("/com/system76/SnapshotDaemon", service)
		.await?;

	info!("Starting pop-snapshot daemon");

	std::future::pending::<()>().await;

	Ok(())
}
