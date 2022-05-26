// SPDX-License-Identifier: MPL-2.0

pub mod create;
pub mod list;
pub mod mount;
pub mod restore;

use std::path::Path;
use sys_mount::{Mount, UnmountDrop};
use tempfile::TempDir;

pub struct MountedBtrfs {
	_mount: UnmountDrop<Mount>,
	tempdir: TempDir,
}

impl MountedBtrfs {
	pub fn path(&self) -> &Path {
		self.tempdir.path()
	}
}
