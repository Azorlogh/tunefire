use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Track {
	pub id: Uuid,
	pub source: String,
	pub artist: String,
	pub title: String,
	pub tags: Vec<(String, f32)>,
}

#[derive(Debug, Clone)]
pub struct Tag {
	pub id: Uuid,
	pub name: String,
}
