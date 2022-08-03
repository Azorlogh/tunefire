use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Song {
	pub id: Uuid,
	pub source: String,
	pub title: String,
	pub tags: Vec<(String, f32)>,
}
