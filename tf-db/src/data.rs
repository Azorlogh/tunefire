use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

define_id!(TagId);

#[derive(Debug, Clone)]
pub struct Tag {
	pub name: String,
}

define_id!(TrackId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
	pub source: String,
	pub artists: Vec<String>,
	pub title: String,
	pub tags: HashMap<String, f32>, // TODO use TagId
}

define_id!(PlaylistId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
	pub name: String,
	pub track_ids: Vec<Uuid>,
}
