use std::{rc::Rc, time::Duration};

use anyhow::Result;
use crossbeam_channel::Receiver;
use druid::{
	widget::Controller, Env, Event, EventCtx, ExtEventSink, LifeCycle, LifeCycleCtx, Selector,
	SingleUse, Widget, WidgetId,
};
use tf_player::{
	player::{self},
	SourcePlugin, TrackSource,
};
use tracing::warn;
use url::Url;

use crate::{media_controls::MediaControls, state::Track, State};

pub const PLAYER_CLEAR: Selector = Selector::new("player.clear");
pub const PLAYER_ENQUEUE: Selector<Track> = Selector::new("player.enqueue");
pub const PLAYER_PLAY_PAUSE: Selector = Selector::new("player.play-pause");
pub const PLAYER_SEEK: Selector<Duration> = Selector::new("player.seek");
pub const PLAYER_PREV: Selector = Selector::new("player.prev");
pub const PLAYER_NEXT: Selector = Selector::new("player.next");
pub const PLAYER_EVENT: Selector<player::Event> = Selector::new("player.event");
pub const PLAYER_SET_VOLUME: Selector<f32> = Selector::new("player.set-volume");
pub const PLAYER_CREATED_SOURCE: Selector<(Track, SingleUse<tf_player::TrackSource>)> =
	Selector::new("player.source.created");

pub struct PlaybackController {
	player: player::Controller,
	event_receiver: Option<Receiver<player::Event>>,
	media_controls: Option<MediaControls>,
}

impl PlaybackController {
	pub fn new() -> Result<Self> {
		let (player, events) = player::Player::spawn()?;

		Ok(Self {
			player,
			event_receiver: Some(events),
			media_controls: None,
		})
	}

	pub fn spawn_event_thread(&mut self, sink: ExtEventSink) {
		let events = self.event_receiver.take();
		std::thread::Builder::new()
			.name(String::from("event sink"))
			.spawn(move || {
				for event in events.unwrap() {
					sink.submit_command(PLAYER_EVENT, Box::new(event), druid::Target::Global)
						.unwrap();
				}
			})
			.unwrap();
	}

	pub fn request_track_audio_source(
		&mut self,
		data: &mut State,
		track: &Track,
		sink: ExtEventSink,
		widget_id: WidgetId,
	) {
		let plugins: Vec<Box<dyn SourcePlugin>> = data
			.plugins
			.iter()
			.filter_map(|p| p.read().get_source_plugin())
			.collect();
		let url = Url::parse(&track.source).unwrap();
		let track = track.clone();
		std::thread::spawn(move || {
			if let Some(result) = plugins.iter().find_map(|p| p.handle_url(&url)) {
				match result {
					Ok(source) => sink
						.submit_command(
							PLAYER_CREATED_SOURCE,
							Box::new((track, SingleUse::new(source))),
							widget_id,
						)
						.unwrap(),
					Err(e) => {
						warn!("error while handling track {url:?}: {e}");
					}
				}
			} else {
				warn!("no plugin could handle the track: {url:?}");
			}
		});
	}

	pub fn queue_track(
		&mut self,
		data: &mut State,
		track: &Track,
		track_source: tf_player::TrackSource,
	) {
		self.player.queue_track(track_source).unwrap();
		data.current_track = Some(track.clone());
		self.update_media_controls(data);
		self.play();
	}

	pub fn update_media_controls(&mut self, data: &State) {
		match &data.current_track {
			Some(track) => {
				self.media_controls
					.as_mut()
					.map(|c| c.set_metadata(&track.format_artists(), &track.title));

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
}

impl<W: Widget<State>> Controller<State, W> for PlaybackController {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut EventCtx,
		event: &Event,
		data: &mut State,
		env: &Env,
	) {
		let handled = match event {
			Event::Command(cmd) => match cmd {
				_ if cmd.is(PLAYER_EVENT) => {
					match cmd.get_unchecked::<player::Event>(PLAYER_EVENT) {
						player::Event::StateChanged(ps) => {
							let until_empty = ps
								.get_playing()
								.map(|p| p.track.duration - p.offset)
								.unwrap_or_default() + Duration::from_secs(
								self.player.nb_queued() as u64 * 10000000,
							);
							if until_empty < Duration::from_secs(3) {
								if let Some(track) = data.queue.front() {
									self.request_track_audio_source(
										data,
										&track.clone(),
										ctx.get_external_handle(),
										ctx.widget_id(),
									)
								}
							}
							data.player_state = Rc::new(ps.clone());
						}
						player::Event::TrackEnd => {
							data.history.push_front(data.current_track.take().unwrap());
							if let Some(track) = data.queue.pop_front() {
								data.current_track = Some(track);
								self.update_media_controls(data);
							} else {
								data.current_track = None;
								self.update_media_controls(data);
							}
						}
					}
					druid::Handled::No
				}
				_ if cmd.is(PLAYER_CLEAR) => {
					self.player.clear();
					druid::Handled::Yes
				}
				_ if cmd.is(PLAYER_ENQUEUE) => {
					let track = cmd.get_unchecked::<Track>(PLAYER_ENQUEUE);
					self.request_track_audio_source(
						data,
						track,
						ctx.get_external_handle(),
						ctx.widget_id(),
					);
					druid::Handled::Yes
				}
				_ if cmd.is(PLAYER_PLAY_PAUSE) => {
					self.play_pause(data);
					druid::Handled::Yes
				}
				_ if cmd.is(PLAYER_PREV) && self.player.nb_queued() == 0 => {
					if let Some(track) = data.history.pop_front() {
						self.request_track_audio_source(
							data,
							&track,
							ctx.get_external_handle(),
							ctx.widget_id(),
						);
						data.queue.push_front(data.current_track.take().unwrap());
						data.current_track = Some(track);
						self.player.skip().unwrap();
						self.update_media_controls(data);
					}
					druid::Handled::Yes
				}
				_ if cmd.is(PLAYER_NEXT) => {
					if self.player.nb_queued() == 0 {
						if let Some(track) = data.queue.pop_front() {
							self.request_track_audio_source(
								data,
								&track,
								ctx.get_external_handle(),
								ctx.widget_id(),
							);
							data.history.push_front(data.current_track.take().unwrap());
							data.current_track = Some(track);
							self.player.skip().unwrap();
							self.update_media_controls(data);
						}
					}
					druid::Handled::Yes
				}
				_ if cmd.is(PLAYER_SEEK) => {
					let pos = cmd.get_unchecked::<Duration>(PLAYER_SEEK);
					self.player.seek(*pos).unwrap();
					druid::Handled::Yes
				}
				_ if cmd.is(PLAYER_SET_VOLUME) => {
					let volume = cmd.get_unchecked::<f32>(PLAYER_SET_VOLUME);
					self.player.set_volume(*volume).unwrap();
					druid::Handled::Yes
				}
				_ if cmd.is(PLAYER_CREATED_SOURCE) => {
					let (track, source) =
						cmd.get_unchecked::<(Track, SingleUse<TrackSource>)>(PLAYER_CREATED_SOURCE);
					self.queue_track(data, track, source.take().unwrap());
					druid::Handled::Yes
				}
				_ => druid::Handled::No,
			},
			_ => druid::Handled::No,
		};

		if handled.is_handled() {
			ctx.set_handled();
		}

		child.event(ctx, event, data, env);
	}

	fn lifecycle(
		&mut self,
		child: &mut W,
		ctx: &mut LifeCycleCtx,
		event: &LifeCycle,
		data: &State,
		env: &Env,
	) {
		if let LifeCycle::WidgetAdded = event {
			self.spawn_event_thread(ctx.get_external_handle());
			// TODO: self.spawn_source_thread();
			self.media_controls = MediaControls::new(ctx.window()).ok();
		}
		child.lifecycle(ctx, event, data, env)
	}
}
