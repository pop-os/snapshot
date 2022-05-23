// SPDX-License-Identifier: MPL-2.0
mod snapshot;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
	let x = snapshot::mount_snapshot_folder()?;
	x.make_snapshot()?;
	Ok(())
}
