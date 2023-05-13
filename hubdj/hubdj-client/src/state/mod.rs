use std::rc::Rc;

use druid::{im, ArcStr, Data, Lens};
use hubdj_core::{UserId, UserToken};

#[derive(Clone, Data)]
pub enum State {
	Disconnected(StateDisconnected),
	Connected(StateConnected),
}

impl Default for State {
	fn default() -> Self {
		State::Disconnected(StateDisconnected {
			name: String::new(),
		})
	}
}

#[derive(Clone, Data, Lens)]
pub struct StateDisconnected {
	pub name: String,
}

#[derive(Clone, Data, Lens)]
pub struct StateConnected {
	pub id: Rc<UserId>,
	pub token: Rc<UserToken>,
	pub name: String,
	pub booth: Option<Booth>,
	pub users: im::OrdMap<Rc<UserId>, UserState>,
	pub in_queue: bool,
	pub tracklist: Tracklist,
}

#[derive(Clone, Data, Lens)]
pub struct Tracklist {
	pub query: String,
	pub tracks: im::Vector<Track>,
}

#[derive(Clone, Data, Lens)]
pub struct Booth {
	pub dj: Rc<UserId>,
	pub song: Song,
}

#[derive(Clone, Data, Lens)]
pub struct Song {
	pub url: String,
	pub artist: String,
	pub title: String,
}

#[derive(Clone, Data)]
pub enum UserState {
	Loading,
	Loaded(User),
}

#[derive(Clone, Data, Lens)]
pub struct User {
	pub id: Rc<UserId>,
	pub name: String,
	pub queue: Option<im::Vector<String>>,
}

#[derive(Clone, Data, Lens)]
pub struct Track {
	// pub id: Arc<Uuid>,
	pub source: ArcStr,
	pub title: ArcStr,
	pub artists: String,
	// pub tags: im::HashMap<ArcStr, f32>,
}
