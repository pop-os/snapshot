// SPDX-License-Identifier: MPL-2.0
pub(crate) mod service;
pub(crate) mod snapshot;
pub(crate) mod util;

#[macro_use]
extern crate tracing;

use anyhow::{Context, Result};
use std::future::pending;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use zbus::ConnectionBuilder;

#[tokio::main]
async fn main() -> Result<()> {
	// Set up the tracing logger.
	tracing_subscriber::registry()
		.with(fmt::layer())
		.with(
			EnvFilter::builder()
				.with_default_directive(LevelFilter::INFO.into())
				.from_env_lossy(),
		)
		.init();

	let service = service::SnapshotService;
	let _ = ConnectionBuilder::system()
		.context("failed to get system dbus connection")?
		.name("com.system76.SnapshotDaemon")?
		.serve_at("/com/system76/SnapshotDaemon", service)?
		.build()
		.await?;

	info!("Starting pop-snapshot daemon");
	pending::<()>().await;

	Ok(())
}
