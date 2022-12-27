use std::{rc::Rc, sync::Arc};

use anyhow::Result;
use druid::{im, Data, Lens};
use parking_lot::RwLock;
use tf_player::player;
use uuid::Uuid;

mod track;
pub use track::Track;

mod track_edit;
pub use track_edit::{TagSuggestions, TrackEdit};

mod track_new;
pub use track_new::NewTrack;

use crate::plugins::{self, Plugin, SearchResult};

#[derive(Clone, Data, Lens)]
pub struct State {
	pub plugins: im::Vector<Arc<RwLock<Box<dyn Plugin>>>>,
	pub tracks: im::Vector<Track>,
	pub shown_tags: im::Vector<String>,
	#[data(same_fn = "PartialEq::eq")]
	pub player_state: Rc<player::State>,
	pub queue: im::Vector<Track>,
	pub history: im::Vector<Track>,
	pub query: String,
	pub new_track: Option<NewTrack>,
	pub new_track_search: String,
	pub track_search_results: TrackSuggestions,
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

		let sc: Box<dyn Plugin> = Box::new(plugins::Soundcloud::new().unwrap());
		Ok(Self {
			plugins: im::Vector::from_iter([Arc::new(RwLock::new(sc))].into_iter()),

			tracks,
			shown_tags: im::Vector::new(),
			player_state: Rc::new(player::State::default()),
			queue: im::Vector::new(),
			history: im::Vector::new(),
			query: String::new(),
			new_track: None,
			new_track_search: String::new(),
			track_search_results: TrackSuggestions {
				tracks: im::Vector::new(),
				selected: 0,
			},
			track_edit: None,
			current_track: None,
			selected_track: None,
			volume: 1.0,
		})
	}
}

#[derive(Clone, Data, Lens, Debug)]
pub struct TrackSuggestions {
	pub tracks: im::Vector<SearchResult>,
	pub selected: usize,
}
