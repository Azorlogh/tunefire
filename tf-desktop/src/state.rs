use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;
use druid::{im, Data, Lens};
use tf_db::Track;
use tf_player::player;
use uuid::Uuid;

#[derive(Clone, Data, Lens)]
pub struct State {
	pub tracks: im::Vector<Rc<RefCell<Track>>>,
	pub shown_tags: im::Vector<String>,
	#[data(same_fn = "PartialEq::eq")]
	pub player_state: Rc<player::State>,
	pub queue: im::Vector<Rc<Track>>,
	pub history: im::Vector<Rc<Track>>,
	pub query: String,
	pub new_track: Option<NewTrack>,
	pub new_track_url: String,
	pub track_edit: Option<TrackEdit>,
	pub current_track: Option<Rc<Track>>,
	pub selected_track: Option<Rc<Uuid>>,
	pub volume: f64,
}

impl State {
	pub fn new(db: &mut tf_db::Client) -> Result<Self> {
		let tracks: im::Vector<_> = db
			.list_filtered(&"".parse::<tf_db::Filter>().unwrap())?
			.iter()
			.cloned()
			.map(RefCell::new)
			.map(Rc::new)
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

#[derive(Clone, Default, Data, Lens)]
pub struct NewTrack {
	pub source: String,
	pub title: String,
	pub artist: String,
}

#[derive(Clone, Data, Lens)]
pub struct TrackEdit {
	pub id: Rc<Uuid>,
	pub title: String,
	pub source: String,
	pub tags: im::Vector<(String, f32)>,
	pub tag_suggestions: TagSuggestions,
}

#[derive(Clone, Data, Lens, Debug)]
pub struct TagSuggestions {
	pub tags: im::Vector<String>,
	pub selected: usize,
}

impl TrackEdit {
	pub fn new(track: Track) -> Self {
		Self {
			id: Rc::new(track.id),
			title: track.title,
			source: track.source,
			tags: im::Vector::from_iter(track.tags.clone()),
			tag_suggestions: TagSuggestions {
				tags: im::Vector::new(),
				selected: 0,
			},
		}
	}

	pub fn get_tags(&self) -> HashMap<String, f32> {
		self.tags
			.iter()
			.filter(|t| !t.0.is_empty())
			.cloned()
			.collect()
	}
}

#[derive(Clone, Data, Lens)]
pub struct TrackListItem {
	pub track: Rc<Track>,
}
