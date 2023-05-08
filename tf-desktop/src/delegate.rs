use std::sync::Arc;

use anyhow::Result;
use druid::{im, AppDelegate};
use rand::seq::SliceRandom;
use tracing::error;
use uuid::Uuid;

use crate::{command, controller::playback, state::TrackEdit, State};

pub struct Delegate {
	db: tf_db::Client,
}

impl Delegate {
	pub fn new(db: tf_db::Client) -> Result<Self> {
		Ok(Self { db })
	}

	fn apply_track_edit(&mut self, edit: TrackEdit) -> Result<()> {
		self.db.set_track(*edit.id, &edit.get_track())?;
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
							data.tracks = tracks.iter().cloned().map(Into::into).collect();
							data.shown_tags = filter.get_tag_set().into_iter().collect();
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
						Ok(mut tracks) => {
							tracks.shuffle(&mut rand::thread_rng());
							ctx.submit_command(playback::PLAYER_CLEAR);
							data.queue = tracks.iter().cloned().map(Into::into).collect();
							ctx.submit_command(
								playback::PLAYER_ENQUEUE
									.with((data.queue.pop_front().unwrap()).clone()),
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
				data.selected_track = Some(Arc::new(*id));
				if let Some(track_edit) = data.track_edit.take() {
					self.apply_track_edit(track_edit).unwrap();
				}
				if let Ok(track) = self.db.get_track(*id) {
					data.track_edit = Some(TrackEdit::new(*id, track));
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::UI_TRACK_EDIT_CLOSE) => {
				data.selected_track = None;
				if let Some(track_edit) = data.track_edit.take() {
					self.apply_track_edit(track_edit).unwrap();
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::UI_TRACK_IMPORT_OPEN) => {
				let track_import = cmd.get_unchecked::<_>(command::UI_TRACK_IMPORT_OPEN);
				data.track_import = Some(track_import.clone());
				druid::Handled::Yes
			}
			_ if cmd.is(command::UI_TRACK_ADD_CLOSE) => {
				data.track_import = None;
				druid::Handled::Yes
			}

			// db
			_ if cmd.is(command::TRACK_ADD) => {
				let track = cmd.get_unchecked::<tf_db::Track>(command::TRACK_ADD);
				match self.db.add_track(track) {
					Ok(id) => {
						let track = self.db.get_track(id).unwrap();
						data.tracks.push_back((id, track).into());
						data.new_track_search = String::new();
						data.track_import = None;
					}
					Err(e) => error!("{:?}", e),
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::TRACK_DELETE) => {
				let id = cmd.get_unchecked::<Uuid>(command::TRACK_DELETE);
				if let Ok(()) = self.db.delete_track(*id) {
					data.tracks.retain(|track| *track.id != *id);
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::TRACK_EDIT_TAG) => {
				let (track, tag, value) = cmd.get_unchecked(command::TRACK_EDIT_TAG);
				if let Err(e) = self.db.set_tag(*track, tag, *value) {
					error!("{e}");
				}
				druid::Handled::Yes
			}
			_ if cmd.is(command::TAG_SEARCH) => {
				let q = cmd.get_unchecked::<String>(command::TAG_SEARCH);
				if q != "" {
					let results = self.db.search_tag(q, 3).unwrap();
					data.track_edit.as_mut().unwrap().tag_suggestions.tags =
						im::Vector::from_iter(results.into_iter().map(|(tag, _)| tag));
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
