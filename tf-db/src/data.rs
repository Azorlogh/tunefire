use std::collections::HashMap;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Track {
	pub id: Uuid,
	pub source: String,
	pub artist: String,
	pub title: String,
	pub tags: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct Tag {
	pub id: Uuid,
	pub name: String,
}
