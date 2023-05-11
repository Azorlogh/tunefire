use druid::{im, Data, Lens};

use super::TagSuggestions;
use crate::widget::common::smart_list::IdentifiedVector;

#[derive(Clone, Data)]
pub enum TrackImport {
	Single(NewTrack),
	Bulk(NewTrackBulk),
}

#[derive(Clone, Default, Data, Lens)]
pub struct NewTrackBulk {
	pub tracks: im::Vector<NewTrack>,
	pub tags: IdentifiedVector<(String, f32)>,
	pub tag_suggestions: TagSuggestions,
}

#[derive(Clone, Default, Data, Lens)]
pub struct NewTrack {
	pub source: String,
	pub title: String,
	pub artists: IdentifiedVector<String>,
	pub tags: IdentifiedVector<(String, f32)>,
}

impl NewTrack {
	pub fn get_track(&self) -> tf_db::Track {
		tf_db::Track {
			source: self.source.clone(),
			artists: self.artists.iter().map(|(_, name)| name).cloned().collect(),
			title: self.title.clone(),
			tags: self.tags.iter().map(|(_, tag)| tag).cloned().collect(),
		}
	}
}
