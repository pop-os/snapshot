// SPDX-License-Identifier: MPL-2.0
mod service;
mod snapshot;

use anyhow::{Context, Result};
use std::future::pending;
use zbus::ConnectionBuilder;

#[tokio::main]
async fn main() -> Result<()> {
	let service = service::SnapshotService;
	let _ = ConnectionBuilder::system()
		.context("failed to get system dbus connection")?
		.name("com.system76.SnapshotDaemon")?
		.serve_at("/com/system76/SnapshotDaemon", service)?
		.build()
		.await?;

	pending::<()>().await;

	Ok(())
}
