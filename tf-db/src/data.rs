use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
	pub source: String,
	pub artists: Vec<String>,
	pub title: String,
	pub tags: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct Tag {
	pub id: Uuid,
	pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
	pub name: String,
	pub tracks: Vec<Uuid>,
}
