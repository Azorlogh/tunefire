use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use druid::AppDelegate;
use tracing::error;
use uuid::Uuid;

use crate::{
	command,
	controller::playback,
	state::{NewTrack, TrackEdit},
	State,
};

pub struct Delegate {
	db: tf_db::Client,
}

impl Delegate {
	pub fn new(db: tf_db::Client) -> Result<Self> {
		Ok(Self { db })
	}

	fn apply_track_edit(&mut self, edit: TrackEdit) -> Result<()> {
		self.db.set_tags(*edit.id, &edit.get_tags())?;
		Ok(())
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
		ctx: &mut druid::DelegateCtx,
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
								.cloned()
								.map(RefCell::new)
								.map(Rc::new)
								.collect();
						}
						Err(e) => println!("error while querying {:?}", e),
					},
					Err(e) => println!("invalid query {e:?}"),
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::QUERY_PLAY) => {
				match data.query.parse::<tf_db::Filter>() {
					Ok(filter) => match self.db.list_filtered(&filter) {
						Ok(tracks) => {
							ctx.submit_command(playback::PLAYER_CLEAR);
							data.queue = tracks.iter().cloned().map(Rc::new).collect();
							ctx.submit_command(
								playback::PLAYER_ENQUEUE
									.with((*data.queue.pop_front().unwrap()).clone()),
							);
						}
						Err(e) => println!("error while querying {:?}", e),
					},
					Err(e) => println!("invalid query {e:?}"),
				}
				druid::Handled::Yes
			}

			// ui
			_ if cmd.is(command::UI_TRACK_EDIT_OPEN) => {
				let id = cmd.get::<Uuid>(command::UI_TRACK_EDIT_OPEN).unwrap();
				data.selected_track = Some(Rc::new(*id));
				if let Ok(track) = self.db.get_track(*id) {
					data.track_edit = Some(TrackEdit::new(track));
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::UI_TRACK_EDIT_CLOSE) => {
				if let Some(track_edit) = data.track_edit.take() {
					self.apply_track_edit(track_edit).unwrap();
				}
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
						data.tracks.push_back(Rc::new(RefCell::new(track)));
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
					data.tracks.retain(|track| track.borrow().id != *id);
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::TAG_SEARCH) => {
				let q = cmd.get::<String>(command::TAG_SEARCH).unwrap();
				if q != "" {
					let results = self.db.search_tag(q).unwrap();
					println!("{:?}", results);
					data.track_edit.as_mut().unwrap().tag_suggestions.tags = results.into();
				}
				druid::Handled::Yes
			}
			_ => druid::Handled::No,
		}
	}

	fn window_added(
		&mut self,
		_id: druid::WindowId,
		_handle: druid::WindowHandle,
		_data: &mut State,
		_env: &druid::Env,
		_ctx: &mut druid::DelegateCtx,
	) {
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
