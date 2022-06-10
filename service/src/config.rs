// SPDX-License-Identifier: MPL-2.0
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
	/// The path (relative to the base subvolume of the btrfs partition)
	/// to the directory containing the snapshots.
	///
	/// Defaults to `@snapshots/pop-snapshots`.
	pub snapshot_path: PathBuf,
	/// A list of subvolumes to exclude by default.
	///
	/// `@snapshots` will *always* be excluded, regardless of this list.
	pub exclude_subvolumes: Vec<String>,
	/// A list of subvolumes to include by default.
	///
	/// This will take precedence over `subvolumes_to_exclude` if both are
	/// specified.
	pub include_subvolumes: Option<Vec<String>>,
	/// The logging filter to use.
	///
	/// Can be any [`EnvFilter`](https://docs.rs/tracing-subscriber/0.3.11/tracing_subscriber/filter/struct.EnvFilter.html#directives)
	/// compatible string.
	///
	/// The `RUST_LOG` environment variable will override this config option.
	///
	/// Defaults to "info".
	pub log_level: String,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			snapshot_path: "@snapshots/pop-snapshots".into(),
			exclude_subvolumes: vec!["@home".into()],
			include_subvolumes: None,
			log_level: "info".into(),
		}
	}
}
