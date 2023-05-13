use std::sync::Arc;

use druid::{Data, Lens};
use uuid::Uuid;

#[derive(Clone, Data, Lens)]
pub struct Playlist {
	pub id: Arc<Uuid>,
	pub name: String,
}

impl From<(Uuid, tf_db::Playlist)> for Playlist {
	fn from((id, p): (Uuid, tf_db::Playlist)) -> Self {
		Self {
			id: Arc::new(id),
			name: p.name,
		}
	}
}
