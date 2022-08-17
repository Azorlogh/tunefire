use std::{rc::Rc, time::Duration};

use anyhow::Result;
use druid::{im, AppDelegate};
use souvlaki::MediaControlEvent;
use tf_db::Track;
use tf_player::player::{self, Player};
use tracing::{error, warn};
use url::Url;
use uuid::Uuid;

use crate::{
	command,
	media_controls::MediaControls,
	state::{NewTrack, TrackEdit, TrackListItem},
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

	pub fn queue_track(&mut self, data: &mut State, track: Track) {
		self.player
			.queue_track(Url::parse(&track.source).unwrap())
			.unwrap();
		data.current_track = Some(Rc::new(track));
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
		match &data.current_track {
			Some(track) => {
				self.media_controls
					.as_mut()
					.map(|c| c.set_metadata(&track.artist, &track.title));

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
						Ok(tracks) => {
							data.tracks = tracks
								.iter()
								.map(|s| TrackListItem {
									track: Rc::new(s.clone()),
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
				let mut queue: im::Vector<Rc<Track>> = data
					.tracks
					.iter()
					.filter_map(|item| self.db.get_track(item.track.id).ok())
					.map(|s| Rc::new(s))
					.collect();
				if let Some(track) = queue.pop_front() {
					self.queue_track(data, (*track).clone());
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
				// queue next track
				let until_empty = ps
					.get_playing()
					.map(|p| p.track.duration - p.offset)
					.unwrap_or_default() + Duration::from_secs(
					self.player.nb_queued() as u64 * 10000000,
				);
				if until_empty < Duration::from_secs(1) {
					if let Some(track) = data.queue.pop_front() {
						self.player
							.queue_track(Url::parse(&track.source).unwrap())
							.unwrap();
						data.current_track = Some(track);
						self.update_media_controls(data);
					} else {
						data.current_track = None;
						self.update_media_controls(data);
					}
				}
				data.player_state = Rc::new(ps);
				druid::Handled::Yes
			}
			_ if cmd.is(command::TRACK_PLAY) => {
				let id = cmd.get::<Uuid>(command::TRACK_PLAY).unwrap();
				let track = self.db.get_track(*id).unwrap();
				self.queue_track(data, track);
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
			_ if cmd.is(command::UI_TRACK_EDIT_OPEN) => {
				let id = cmd.get::<Uuid>(command::UI_TRACK_EDIT_OPEN).unwrap();
				data.tracks.iter_mut().for_each(|item| {
					item.selected = item.track.id == *id;
				});
				if let Ok(track) = self.db.get_track(*id) {
					data.track_edit = Some(TrackEdit::new(track));
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::UI_TRACK_EDIT_CLOSE) => {
				data.track_edit = None;
				druid::Handled::Yes
			}
			_ if cmd.is(command::UI_TRACK_ADD_OPEN) => {
				let source = cmd.get::<String>(command::UI_TRACK_ADD_OPEN).unwrap();
				data.new_track = Some(NewTrack {
					source: source.clone(),
					title: String::new(),
					artist: String::new(),
				});
				druid::Handled::Yes
			}
			_ if cmd.is(command::UI_TRACK_ADD_CLOSE) => {
				data.new_track = None;
				druid::Handled::Yes
			}

			// db
			_ if cmd.is(command::TRACK_ADD) => {
				let NewTrack {
					source,
					artist,
					title,
				} = cmd.get::<NewTrack>(command::TRACK_ADD).unwrap();
				match self.db.add_track(source, artist, title) {
					Ok(id) => {
						let track = self.db.get_track(id).unwrap();
						data.tracks.push_back(TrackListItem {
							track: Rc::new(track),
							selected: true,
						});
						data.new_track_url = String::new();
						data.new_track = None;
					}
					Err(e) => error!("{:?}", e),
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::TRACK_DELETE) => {
				let id = cmd.get::<Uuid>(command::TRACK_DELETE).unwrap();
				if let Ok(()) = self.db.delete_track(*id) {
					data.tracks.retain(|item| item.track.id != *id);
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::TAG_ADD) => {
				cmd.get::<Uuid>(command::TAG_ADD).unwrap();
				data.track_edit
					.as_mut()
					.unwrap()
					.tags
					.push_back((String::new(), 0.5));
				druid::Handled::Yes
			}
			_ if cmd.is(command::TAG_RENAME) => {
				let track_edit = data.track_edit.as_mut().unwrap();
				let (from, to) = cmd.get::<(String, String)>(command::TAG_RENAME).unwrap();
				if from != "" {
					self.db.set_tag(*track_edit.id, from, 0.0).unwrap();
				}
				let tag = track_edit.tags.iter().find(|tag| &tag.0 == to).unwrap();
				self.db.set_tag(*track_edit.id, to, tag.1).unwrap();
				druid::Handled::Yes
			}
			_ if cmd.is(command::TAG_TWEAK) => {
				let track_edit = data.track_edit.as_mut().unwrap();
				let (name, value) = cmd.get::<(String, f32)>(command::TAG_TWEAK).unwrap();
				if name != "" {
					self.db.set_tag(*track_edit.id, name, *value).unwrap();
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::TAG_DELETE) => {
				let tag_name = cmd.get::<String>(command::TAG_DELETE).unwrap();
				let track_edit = data.track_edit.as_mut().unwrap();
				match track_edit.tags.iter().find(|tag| &tag.0 == tag_name) {
					Some((name, _)) => {
						self.db.set_tag(*track_edit.id, &name, 0.0).unwrap();
					}
					_ => {}
				}
				track_edit.tags.retain(|item| &item.0 != tag_name);
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
