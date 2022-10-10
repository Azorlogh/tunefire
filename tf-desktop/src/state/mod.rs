use std::rc::Rc;

use anyhow::Result;
use druid::{im, Data, Lens};
use tf_player::player;
use uuid::Uuid;

mod track;
pub use track::Track;

mod track_edit;
pub use track_edit::{TagSuggestions, TrackEdit};

mod track_new;
pub use track_new::NewTrack;

#[derive(Clone, Data, Lens)]
pub struct State {
	pub tracks: im::Vector<Track>,
	pub shown_tags: im::Vector<String>,
	#[data(same_fn = "PartialEq::eq")]
	pub player_state: Rc<player::State>,
	pub queue: im::Vector<Track>,
	pub history: im::Vector<Track>,
	pub query: String,
	pub new_track: Option<NewTrack>,
	pub new_track_url: String,
	pub track_edit: Option<TrackEdit>,
	pub current_track: Option<Track>,
	pub selected_track: Option<Rc<Uuid>>,
	pub volume: f64,
}

impl State {
	pub fn new(db: &mut tf_db::Client) -> Result<Self> {
		let tracks: im::Vector<_> = db
			.list_filtered(&"".parse::<tf_db::Filter>().unwrap())?
			.iter()
			.cloned()
			.map(Into::into)
			.collect();

		Ok(Self {
			tracks,
			shown_tags: im::Vector::new(),
			player_state: Rc::new(player::State::default()),
			queue: im::Vector::new(),
			history: im::Vector::new(),
			query: String::new(),
			new_track: None,
			new_track_url: String::new(),
			track_edit: None,
			current_track: None,
			selected_track: None,
			volume: 1.0,
		})
	}
}
