use druid::{im, Data, Lens};

#[derive(Clone, Default, Data, Lens)]
pub struct NewTrack {
	pub source: String,
	pub title: String,
	pub artists: im::Vector<String>,
}
