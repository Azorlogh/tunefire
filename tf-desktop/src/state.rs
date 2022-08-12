use std::rc::Rc;

use anyhow::Result;
use druid::{im, Data, Lens};
use tf_db::Song;
use tf_player::player;
use uuid::Uuid;

#[derive(Clone, Data, Lens)]
pub struct State {
	pub songs: im::Vector<SongListItem>,
	#[data(same_fn = "PartialEq::eq")]
	pub player_state: Rc<player::State>,
	pub queue: im::Vector<Rc<Song>>,
	pub query: String,
	pub new_song: Option<NewSong>,
	pub new_song_url: String,
	pub song_edit: Option<SongEdit>,
	pub current_song: Option<Rc<Song>>,
}

impl State {
	pub fn new(db: &mut tf_db::Client) -> Result<Self> {
		let songs: im::Vector<_> = db
			.list()?
			.iter()
			.map(|s| SongListItem {
				song: Rc::new(s.clone()),
				selected: false,
			})
			.collect();

		Ok(Self {
			player_state: Rc::new(player::State::default()),
			songs,
			queue: im::Vector::new(),
			query: String::new(),
			new_song: None,
			new_song_url: String::new(),
			song_edit: None,
			current_song: None,
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
pub struct NewSong {
	pub source: String,
	pub title: String,
	pub artist: String,
}

#[derive(Clone, Data, Lens)]
pub struct SongEdit {
	pub id: Rc<Uuid>,
	pub title: String,
	pub source: String,
	pub tags: im::Vector<(String, f32)>,
}

impl SongEdit {
	pub fn new(song: Song) -> Self {
		Self {
			id: Rc::new(song.id),
			title: song.title,
			source: song.source,
			tags: im::Vector::from_iter(song.tags.clone()),
		}
	}
}

#[derive(Clone, Data, Lens)]
pub struct SongListItem {
	pub selected: bool,
	pub song: Rc<Song>,
}
