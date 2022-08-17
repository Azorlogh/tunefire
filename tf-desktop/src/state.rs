use std::rc::Rc;

use anyhow::Result;
use druid::{im, Data, Lens};
use tf_db::Track;
use tf_player::player;
use uuid::Uuid;

#[derive(Clone, Data, Lens)]
pub struct State {
	pub tracks: im::Vector<TrackListItem>,
	#[data(same_fn = "PartialEq::eq")]
	pub player_state: Rc<player::State>,
	pub queue: im::Vector<Rc<Track>>,
	pub query: String,
	pub new_track: Option<NewTrack>,
	pub new_track_url: String,
	pub track_edit: Option<TrackEdit>,
	pub current_track: Option<Rc<Track>>,
}

impl State {
	pub fn new(db: &mut tf_db::Client) -> Result<Self> {
		let tracks: im::Vector<_> = db
			.list()?
			.iter()
			.map(|s| TrackListItem {
				track: Rc::new(s.clone()),
				selected: false,
			})
			.collect();

		Ok(Self {
			player_state: Rc::new(player::State::default()),
			tracks,
			queue: im::Vector::new(),
			query: String::new(),
			new_track: None,
			new_track_url: String::new(),
			track_edit: None,
			current_track: None,
		})
	}

	// pub fn playing_lens() -> druid::lens::Map<
	// 	impl Fn(&Rc<player::State>) -> Option<Rc<player::state::Playing>>,
	// 	impl Fn(&mut Rc<player::State>, Option<Rc<player::state::Playing>>),
	// > {
	// 	druid::lens::Map::new(
	// 		|s: &Rc<player::State>| s.get_playing().map(|s| Rc::new(s.clone())),
	// 		|s: &mut Rc<player::State>, inner: Option<Rc<player::state::Playing>>| {
	// 			*s = Rc::new(
	// 				inner
	// 					.map(|s| player::State::Playing((*s).clone()))
	// 					.unwrap_or(player::State::Idle),
	// 			);
	// 		},
	// 	)
	// }
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
}

impl TrackEdit {
	pub fn new(track: Track) -> Self {
		Self {
			id: Rc::new(track.id),
			title: track.title,
			source: track.source,
			tags: im::Vector::from_iter(track.tags.clone()),
		}
	}
}

#[derive(Clone, Data, Lens)]
pub struct TrackListItem {
	pub selected: bool,
	pub track: Rc<Track>,
}
