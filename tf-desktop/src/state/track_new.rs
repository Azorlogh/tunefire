use druid::{im, Data, Lens};

#[derive(Clone, Data)]
pub enum TrackImport {
	Single(NewTrack),
	Bulk(NewTrackBulk),
}

#[derive(Clone, Default, Data, Lens)]
pub struct NewTrackBulk {
	pub tracks: im::Vector<NewTrack>,
	pub tag: im::Vector<(String, f32)>,
}

#[derive(Clone, Default, Data, Lens)]
pub struct NewTrack {
	pub source: String,
	pub title: String,
	pub artists: im::Vector<String>,
}

impl NewTrack {
	pub fn get_track(&self) -> tf_db::Track {
		tf_db::Track {
			source: self.source.clone(),
			artists: self.artists.iter().cloned().collect(),
			title: self.title.clone(),
			tags: Default::default(),
		}
	}
}
