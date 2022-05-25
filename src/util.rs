// SPDX-License-Identifier: MPL-2.0

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

/// Finds the btrfs partition that contains the root subvolume.
///
/// This works by scanning /proc/mounts for a mount that has the
/// `subvol=/@root` option.
pub async fn find_root_device() -> Result<PathBuf> {
	let mounts = fs::read_to_string("/proc/mounts")
		.await
		.context("failed to read /proc/mounts")?;
	mounts
		.lines()
		.find(|line| line.contains("subvol=/@root"))
		.and_then(|line| line.split_whitespace().next())
		.map(PathBuf::from)
		.context("failed to find @root")
}
