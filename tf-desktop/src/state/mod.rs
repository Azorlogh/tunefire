use std::{rc::Rc, sync::Arc};

use anyhow::Result;
use druid::{im, Data, Lens};
use parking_lot::RwLock;
use tf_player::player;
use uuid::Uuid;

mod track;
pub use track::Track;

mod track_edit;
pub use track_edit::TrackEdit;

mod track_new;
use tf_plugin::{self, Plugin, SearchResult};
pub use track_new::*;

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
	pub track_import: Option<TrackImport>,
	pub new_track_search: String,
	pub track_search_results: TrackSuggestions,
	pub track_edit: Option<TrackEdit>,
	pub current_track: Option<Track>,
	pub selected_track: Option<Arc<Uuid>>,
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

		let mut plugins: Vec<Box<dyn Plugin>> = vec![];
		#[cfg(feature = "local")]
		plugins.push(Box::new(tf_plugin_local::Local));
		#[cfg(feature = "soundcloud")]
		plugins.push(Box::new(tf_plugin_soundcloud::Soundcloud::new().unwrap()));
		#[cfg(feature = "youtube")]
		plugins.push(Box::new(tf_plugin_youtube::Youtube::new().unwrap()));

		Ok(Self {
			plugins: im::Vector::from_iter(plugins.into_iter().map(|p| Arc::new(RwLock::new(p)))),
			tracks,
			shown_tags: im::Vector::new(),
			player_state: Rc::new(player::State::default()),
			queue: im::Vector::new(),
			history: im::Vector::new(),
			query: String::new(),
			track_import: None,
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

#[derive(Clone, Data, Lens, Debug, Default)]
pub struct TagSuggestions {
	pub tags: im::Vector<String>,
	pub selected: usize,
}
