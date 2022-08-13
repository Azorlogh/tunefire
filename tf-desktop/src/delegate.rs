use std::{rc::Rc, time::Duration};

use anyhow::Result;
use druid::{im, AppDelegate};
use souvlaki::MediaControlEvent;
use tf_db::Song;
use tf_player::player::{self, Player};
use tracing::{error, warn};
use url::Url;
use uuid::Uuid;

use crate::{
	command,
	media_controls::MediaControls,
	state::{NewSong, SongEdit, SongListItem},
	State,
};

pub struct Delegate {
	db: tf_db::Client,
	player: player::Controller,
	media_controls: Option<MediaControls>,
}

impl Delegate {
	pub fn new(db: tf_db::Client) -> Result<Self> {
		Ok(Self {
			db,
			player: Player::new()?,
			media_controls: None,
		})
	}

	pub fn queue_song(&mut self, data: &mut State, song: Song) {
		self.player
			.queue_song(Url::parse(&song.source).unwrap())
			.unwrap();
		data.current_song = Some(Rc::new(song));
		self.update_media_controls(data);
		self.play();
	}

	pub fn play_pause(&mut self, data: &State) {
		if let Some(p) = data.player_state.get_playing() {
			if p.paused {
				self.play();
			} else {
				self.pause();
			}
		}
	}

	pub fn play(&mut self) {
		self.player.play().unwrap();
		self.media_controls.as_mut().map(|c| c.set_is_playing(true));
	}

	pub fn pause(&mut self) {
		self.player.pause().unwrap();
		self.media_controls
			.as_mut()
			.map(|c| c.set_is_playing(false));
	}

	pub fn update_media_controls(&mut self, data: &State) {
		match &data.current_song {
			Some(song) => {
				self.media_controls
					.as_mut()
					.map(|c| c.set_metadata(&song.artist, &song.title));

				self.media_controls.as_mut().map(|c| {
					c.set_is_playing(true).ok();
				});
			}
			None => {
				self.media_controls
					.as_mut()
					.map(|c| c.set_is_playing(false));
			}
		}
	}
}

impl AppDelegate<State> for Delegate {
	fn event(
		&mut self,
		_ctx: &mut druid::DelegateCtx,
		_window_id: druid::WindowId,
		event: druid::Event,
		_data: &mut State,
		_env: &druid::Env,
	) -> Option<druid::Event> {
		Some(event)
	}

	fn command(
		&mut self,
		_ctx: &mut druid::DelegateCtx,
		_target: druid::Target,
		cmd: &druid::Command,
		data: &mut State,
		_env: &druid::Env,
	) -> druid::Handled {
		match cmd {
			// query
			_ if cmd.is(command::QUERY_RUN) => {
				match data.query.parse::<tf_db::Filter>() {
					Ok(filter) => match self.db.list_filtered(&filter) {
						Ok(songs) => {
							data.songs = songs
								.iter()
								.map(|s| SongListItem {
									song: Rc::new(s.clone()),
									selected: false,
								})
								.collect();
						}
						Err(e) => println!("error while querying {:?}", e),
					},
					Err(e) => println!("invalid query {e:?}"),
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::QUERY_PLAY) => {
				let mut queue: im::Vector<Rc<Song>> = data
					.songs
					.iter()
					.filter_map(|item| self.db.get_song(item.song.id).ok())
					.map(|s| Rc::new(s))
					.collect();
				if let Some(song) = queue.pop_front() {
					self.queue_song(data, (*song).clone());
				}
				data.queue = queue;
				druid::Handled::No
			}

			// player
			_ if cmd.is(command::PLAYER_TICK) => {
				match &self.media_controls.as_mut().unwrap().events.try_recv() {
					Ok(MediaControlEvent::Play) => self.play(),
					Ok(MediaControlEvent::Pause) => self.pause(),
					Ok(MediaControlEvent::Toggle) => self.play_pause(data),
					_ => {}
				}

				// update player bar
				let ps = (*self.player.state().read()).clone();
				// queue next song
				let until_empty = ps
					.get_playing()
					.map(|p| p.song.duration - p.offset)
					.unwrap_or_default() + Duration::from_secs(
					self.player.nb_queued() as u64 * 10000000,
				);
				if until_empty < Duration::from_secs(1) {
					if let Some(song) = data.queue.pop_front() {
						self.player
							.queue_song(Url::parse(&song.source).unwrap())
							.unwrap();
						data.current_song = Some(song);
						self.update_media_controls(data);
					} else {
						data.current_song = None;
						self.update_media_controls(data);
					}
				}
				data.player_state = Rc::new(ps);
				druid::Handled::Yes
			}
			_ if cmd.is(command::SONG_PLAY) => {
				let id = cmd.get::<Uuid>(command::SONG_PLAY).unwrap();
				let song = self.db.get_song(*id).unwrap();
				self.queue_song(data, song);
				druid::Handled::No
			}
			_ if cmd.is(command::PLAYER_PLAY_PAUSE) => {
				self.play_pause(data);
				druid::Handled::Yes
			}
			_ if cmd.is(command::PLAYER_SEEK) => {
				let pos = cmd.get::<Duration>(command::PLAYER_SEEK).unwrap();
				self.player.seek(*pos).unwrap();
				druid::Handled::Yes
			}

			// ui
			_ if cmd.is(command::UI_SONG_EDIT_OPEN) => {
				let id = cmd.get::<Uuid>(command::UI_SONG_EDIT_OPEN).unwrap();
				data.songs.iter_mut().for_each(|item| {
					item.selected = item.song.id == *id;
				});
				if let Ok(song) = self.db.get_song(*id) {
					data.song_edit = Some(SongEdit::new(song));
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::UI_SONG_EDIT_CLOSE) => {
				data.song_edit = None;
				druid::Handled::Yes
			}
			_ if cmd.is(command::UI_SONG_ADD_OPEN) => {
				let source = cmd.get::<String>(command::UI_SONG_ADD_OPEN).unwrap();
				data.new_song = Some(NewSong {
					source: source.clone(),
					title: String::new(),
					artist: String::new(),
				});
				druid::Handled::Yes
			}
			_ if cmd.is(command::UI_SONG_ADD_CLOSE) => {
				data.new_song = None;
				druid::Handled::Yes
			}

			// db
			_ if cmd.is(command::SONG_ADD) => {
				let NewSong {
					source,
					artist,
					title,
				} = cmd.get::<NewSong>(command::SONG_ADD).unwrap();
				match self.db.add_song(source, artist, title) {
					Ok(id) => {
						let song = self.db.get_song(id).unwrap();
						data.songs.push_back(SongListItem {
							song: Rc::new(song),
							selected: true,
						});
						data.new_song_url = String::new();
						data.new_song = None;
					}
					Err(e) => error!("{:?}", e),
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::SONG_DELETE) => {
				let id = cmd.get::<Uuid>(command::SONG_DELETE).unwrap();
				if let Ok(()) = self.db.delete_song(*id) {
					data.songs.retain(|item| item.song.id != *id);
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::TAG_ADD) => {
				cmd.get::<Uuid>(command::TAG_ADD).unwrap();
				data.song_edit
					.as_mut()
					.unwrap()
					.tags
					.push_back((String::new(), 0.5));
				druid::Handled::Yes
			}
			_ if cmd.is(command::TAG_RENAME) => {
				let song_edit = data.song_edit.as_mut().unwrap();
				let (from, to) = cmd.get::<(String, String)>(command::TAG_RENAME).unwrap();
				if from != "" {
					self.db.set_tag(*song_edit.id, from, 0.0).unwrap();
				}
				let tag = song_edit.tags.iter().find(|tag| &tag.0 == to).unwrap();
				self.db.set_tag(*song_edit.id, to, tag.1).unwrap();
				druid::Handled::Yes
			}
			_ if cmd.is(command::TAG_TWEAK) => {
				let song_edit = data.song_edit.as_mut().unwrap();
				let (name, value) = cmd.get::<(String, f32)>(command::TAG_TWEAK).unwrap();
				if name != "" {
					self.db.set_tag(*song_edit.id, name, *value).unwrap();
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::TAG_DELETE) => {
				let tag_name = cmd.get::<String>(command::TAG_DELETE).unwrap();
				let song_edit = data.song_edit.as_mut().unwrap();
				match song_edit.tags.iter().find(|tag| &tag.0 == tag_name) {
					Some((name, _)) => {
						self.db.set_tag(*song_edit.id, &name, 0.0).unwrap();
					}
					_ => {}
				}
				song_edit.tags.retain(|item| &item.0 != tag_name);
				druid::Handled::Yes
			}
			_ => druid::Handled::No,
		}
	}

	fn window_added(
		&mut self,
		_id: druid::WindowId,
		handle: druid::WindowHandle,
		_data: &mut State,
		_env: &druid::Env,
		_ctx: &mut druid::DelegateCtx,
	) {
		if self.media_controls.is_none() {
			match MediaControls::new(handle) {
				Ok(controls) => self.media_controls = Some(controls),
				Err(err) => warn!("failed to create media controls {}", err),
			}
		}
	}

	fn window_removed(
		&mut self,
		_id: druid::WindowId,
		_data: &mut State,
		_env: &druid::Env,
		_ctx: &mut druid::DelegateCtx,
	) {
	}
}
