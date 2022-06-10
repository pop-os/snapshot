// SPDX-License-Identifier: MPL-2.0
pub(crate) mod config;
pub(crate) mod service;
pub(crate) mod snapshot;
pub(crate) mod util;

#[macro_use]
extern crate tracing;

use crate::service::snapshot::SnapshotObject;
use anyhow::{Context, Result};
use async_signals::Signals;
use futures_util::StreamExt;
use libc::{SIGHUP, SIGTERM};
use std::sync::{
	atomic::{AtomicUsize, Ordering},
	Arc,
};
use tokio::sync::RwLock;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{filter::Directive, fmt, prelude::*, EnvFilter};
use zbus::{zvariant::OwnedObjectPath, ConnectionBuilder, ObjectServer};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

async fn create_new_snapshot(
	object_server: &ObjectServer,
	snapshot_object: SnapshotObject,
) -> Result<OwnedObjectPath> {
	let new_id = COUNTER.fetch_add(1, Ordering::SeqCst);
	let id = OwnedObjectPath::try_from(format!("/com/system76/PopSnapshot/Snapshot/{}", new_id))?;
	object_server
		.at(&id, snapshot_object)
		.await
		.with_context(|| format!("failed to register snapshot {:?}", id))?;
	Ok(id)
}

async fn reload_config(config: Arc<RwLock<config::Config>>) -> Result<()> {
	let mut config = config.write().await;
	*config = tokio::fs::read_to_string("/etc/pop-snapshots.toml")
		.await
		.context("failed to read /etc/pop-snapshots.toml")
		.and_then(|s| toml::from_str::<config::Config>(&s).context("failed to parse config"))?;
	info!("Configuration reloaded");
	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	let config = tokio::fs::read_to_string("/etc/pop-snapshots.toml")
		.await
		.ok()
		.and_then(|s| toml::from_str::<config::Config>(&s).ok())
		.unwrap_or_default();

	let log_level: Directive = config
		.log_level
		.parse()
		.with_context(|| format!("failed to parse log level: {}", config.log_level))?;
	// Set up the tracing logger.
	tracing_subscriber::registry()
		.with(fmt::layer())
		.with(
			EnvFilter::builder()
				.with_default_directive(if cfg!(debug_assertions) {
					LevelFilter::DEBUG.into()
				} else {
					log_level
				})
				.from_env_lossy(),
		)
		.init();

	let config = Arc::new(RwLock::new(config));
	let service = service::SnapshotService::new(config.clone());
	let connection = ConnectionBuilder::system()
		.context("failed to get system dbus connection")?
		.name("com.system76.PopSnapshot")?
		.internal_executor(false)
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
		let mut snapshots_map = service.snapshots.write().await;
		snapshots_map.reserve(snapshots.len());
		for snapshot in snapshots {
			let snapshot_uuid = snapshot.uuid;
			let snapshot_object = SnapshotObject::new(
				snapshot,
				service.snapshots.clone(),
				service.action_lock.clone(),
				config.clone(),
			);
			let id = create_new_snapshot(&*connection.object_server(), snapshot_object)
				.await
				.context("failed to create new snapshot object")?;

			debug!(
				"Created new snapshot object for {} at {:?}",
				snapshot_uuid, id
			);
			snapshots_map.insert(snapshot_uuid, id);
		}
	}
	connection
		.object_server()
		.at("/com/system76/PopSnapshot", service)
		.await?;

	info!("Starting pop-snapshot daemon");

	let mut signals = Signals::new(vec![SIGHUP, SIGTERM])
		.context("failed to create signal handler for SIGHUP+SIGTERM")?;

	tokio::spawn(async move {
		let executor = connection.executor();
		loop {
			executor.tick().await;
		}
	});

	while let Some(signal) = signals.next().await {
		match signal {
			SIGHUP => {
				info!("Received SIGHUP, reloading config");
				match reload_config(config.clone()).await {
					Ok(_) => {
						continue;
					}
					Err(e) => {
						error!("Failed to reload config: {}", e);
						continue;
					}
				}
			}
			SIGTERM => {
				info!("Received SIGTERM, shutting down");
				break;
			}
			_ => continue,
		}
	}

	Ok(())
}
