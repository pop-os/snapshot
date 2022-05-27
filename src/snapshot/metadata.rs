// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct SnapshotMetadata {
	pub uuid: Uuid,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(with = "time::serde::rfc3339")]
	pub creation_time: OffsetDateTime,
	pub subvolumes: Vec<String>,
}

impl SnapshotMetadata {
	pub fn now(
		name: impl Into<Option<String>>,
		description: impl Into<Option<String>>,
		subvolumes: Vec<String>,
	) -> Self {
		SnapshotMetadata {
			uuid: Uuid::new_v4(),
			name: name.into(),
			description: description.into(),
			creation_time: OffsetDateTime::now_utc(),
			subvolumes,
		}
	}
}

impl PartialOrd for SnapshotMetadata {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.creation_time.partial_cmp(&other.creation_time)
	}
}

impl Ord for SnapshotMetadata {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.creation_time.cmp(&other.creation_time)
	}
}
