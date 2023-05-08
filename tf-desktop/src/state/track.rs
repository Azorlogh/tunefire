use std::sync::Arc;

use druid::{im, ArcStr, Data, Lens};
use uuid::Uuid;

#[derive(Clone, Data, Lens)]
pub struct Track {
	pub id: Arc<Uuid>,
	pub source: ArcStr,
	pub title: ArcStr,
	pub artist: ArcStr,
	pub tags: im::HashMap<ArcStr, f32>,
}

impl From<(Uuid, tf_db::Track)> for Track {
	fn from((id, t): (Uuid, tf_db::Track)) -> Self {
		Self {
			id: Arc::new(id),
			source: t.source.into(),
			title: t.title.into(),
			artist: t.artist.into(),
			tags: im::HashMap::from_iter(
				t.tags.iter().map(|(k, &v)| (ArcStr::from(k.to_owned()), v)),
			),
		}
	}
}
