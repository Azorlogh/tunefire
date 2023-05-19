use std::{
	future::Future,
	rc::Rc,
	sync::{Arc, RwLock},
	time::Duration,
};

use anyhow::Result;
use crossbeam_channel::Receiver;
use druid::{
	im, piet::TextStorage, widget::Controller, Event, ExtEventSink, Handled, LifeCycle, Selector,
	SingleUse, Widget, WidgetId,
};
use hubdj_core::{UserId, UserToken};
use tf_player::{player, SourcePlugin};
use tf_plugin::Plugin;
use tokio_stream::StreamExt;
use tonic::{transport::Channel, Response};
use tracing::{error, warn};

use crate::{
	pb::{self, hubdj_client::HubdjClient, AuthRequest, AuthResponse},
	state::{self, State, StateConnected, StateDisconnected, Tracklist, User, UserState},
};

pub const CLIENT_CONNECT_REQ: Selector = Selector::new("client.connect.req");
pub const CLIENT_CONNECT_RES: Selector<SingleUse<Response<AuthResponse>>> =
	Selector::new("client.connect.res");
pub const CLIENT_GET_USER_RES: Selector<SingleUse<Response<pb::User>>> =
	Selector::new("client.get-user.res");
pub const CLIENT_SUBMIT_PLAYLIST_REQ: Selector = Selector::new("client.submit-playlist-req");
pub const CLIENT_JOIN_QUEUE_REQ: Selector = Selector::new("client.join-queue-req");
pub const CLIENT_JOIN_QUEUE_RES: Selector = Selector::new("client.join-queue-res");
pub const CLIENT_LEAVE_QUEUE_REQ: Selector = Selector::new("client.leave-queue-req");
pub const CLIENT_LEAVE_QUEUE_RES: Selector = Selector::new("client.leave-queue-res");
pub const CLIENT_BOOTH_STATE: Selector<pb::Booth> = Selector::new("client.booth-state");
pub const PLAYER_EVENT: Selector<player::Event> = Selector::new("player.event");

pub struct ClientController {
	client: HubdjClient<Channel>,
	player: player::Controller,
	event_receiver: Option<Receiver<player::Event>>,
}

impl ClientController {
	pub async fn new() -> Result<Self> {
		let (player, events) = player::Player::spawn()?;

		Ok(Self {
			client: HubdjClient::connect("http://[::1]:53000").await?,
			player,
			event_receiver: Some(events),
		})
	}

	pub fn spawn_playback_event_thread(&mut self, sink: ExtEventSink) {
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

	pub fn play_track(&mut self, data: &mut StateConnected, url: &str, elapsed: Duration) {
		let plugins: Vec<Box<dyn SourcePlugin>> = data
			.plugins
			.iter()
			.filter_map(|p| p.read().unwrap().get_source_plugin())
			.collect();
		let player = self.player.clone();
		let url = url.to_owned();
		std::thread::spawn(move || {
			if let Some(result) = plugins
				.iter()
				.find_map(|p| p.handle_url(&url.parse().unwrap()))
			{
				match result {
					Ok(source) => {
						player.clear();
						player.queue_track(source).unwrap();
						player.seek(elapsed).unwrap();
					}
					Err(e) => {
						warn!("error while handling track {url:?}: {e}");
					}
				}
			} else {
				warn!("no plugin could handle the track: {url:?}");
			}
		});
	}

	pub fn request<T>(
		&self,
		ctx: &mut druid::EventCtx,
		future: impl FnOnce(ExtEventSink, WidgetId, HubdjClient<Channel>) -> T,
	) where
		T: Future + Send + 'static,
		T::Output: Send + 'static,
	{
		let handle = ctx.get_external_handle();
		let id = ctx.widget_id();
		let client = self.client.clone();
		tokio::spawn(future(handle, id, client));
	}
}

impl<W: Widget<State>> Controller<State, W> for ClientController {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut State,
		env: &druid::Env,
	) {
		let handled = match event {
			Event::Command(cmd) => match cmd {
				_ if cmd.is(CLIENT_CONNECT_REQ) => {
					let State::Disconnected(StateDisconnected{name}) = data else {
						unreachable!()
					};
					let name = name.clone();
					self.request(ctx, |handle, id, mut client| async move {
						let res = client.auth(AuthRequest { name }).await.unwrap();
						handle
							.submit_command(CLIENT_CONNECT_RES, SingleUse::new(res), id)
							.unwrap();
					});
					Handled::Yes
				}
				_ if cmd.is(CLIENT_CONNECT_RES) => {
					let res = cmd
						.get_unchecked::<SingleUse<Response<AuthResponse>>>(CLIENT_CONNECT_RES)
						.take()
						.unwrap();
					let res = res.into_inner();
					let State::Disconnected(StateDisconnected{name}) = data else {
						unreachable!()
					};
					for user in &res.users {
						let user_id = user.clone();
						self.request(ctx, |handle, id, mut client| async move {
							let res = client.get_user(pb::UserId { id: user_id }).await.unwrap();
							handle
								.submit_command(CLIENT_GET_USER_RES, SingleUse::new(res), id)
								.unwrap();
						});
					}
					self.request(ctx, |handle, id, mut client| async move {
						let mut res = client.stream_booth_state(()).await.unwrap().into_inner();
						while let Some(received) = res.next().await {
							handle
								.submit_command(CLIENT_BOOTH_STATE, received.unwrap(), id)
								.unwrap();
						}
					});

					let mut plugins: Vec<Box<dyn Plugin>> = vec![];
					plugins.push(Box::new(tf_plugin_soundcloud::Soundcloud::new().unwrap()));
					plugins.push(Box::new(tf_plugin_youtube::Youtube::new().unwrap()));

					*data = State::Connected(StateConnected {
						id: Rc::new(UserId(res.id)),
						token: Rc::new(UserToken(res.token)),
						name: name.clone(),
						users: im::OrdMap::from_iter(
							res.users
								.into_iter()
								.map(|id| (UserId(id), UserState::Loading)),
						),
						in_queue: false,
						booth: None,
						tracklist: Tracklist {
							query: String::new(),
							tracks: im::Vector::new(),
						},
						queue: Default::default(),
						plugins: im::Vector::from_iter(
							plugins.into_iter().map(|p| Arc::new(RwLock::new(p))),
						),
					});
					Handled::Yes
				}
				_ if cmd.is(CLIENT_BOOTH_STATE) => {
					if let State::Connected(data) = data {
						let booth = cmd.get_unchecked::<pb::Booth>(CLIENT_BOOTH_STATE);
						if let Some(playing) = &booth.playing {
							let url = playing.track.as_ref().map(|t| t.url.clone()).unwrap();
							if data.booth.as_ref().map(|b| b.track.as_ref()) != Some(&url) {
								self.play_track(data, &url, Duration::from_millis(playing.elapsed));
							}
							data.booth = Some(state::Booth {
								dj: Rc::new(UserId(playing.dj)),
								track: Arc::from(playing.track.as_ref().unwrap().url.clone()),
							});
							data.in_queue = playing.dj == data.id.0
								|| playing
									.queue
									.iter()
									.find(|qt| {
										qt.user.as_ref().map(|uid| uid.id) == Some(data.id.0)
									})
									.is_some();
						} else {
							data.in_queue = false;
						}
					}
					Handled::Yes
				}
				_ if cmd.is(CLIENT_GET_USER_RES) => {
					let res = cmd
						.get_unchecked::<SingleUse<Response<pb::User>>>(CLIENT_GET_USER_RES)
						.take()
						.unwrap()
						.into_inner();
					let State::Connected(data) = data else {
						unreachable!();
					};
					let user_state = data.users.get_mut(&Rc::new(UserId(res.id))).unwrap();
					*user_state = UserState::Loaded(User {
						id: Rc::new(UserId(res.id)),
						name: res.name,
						queue: res
							.queue
							.map(|q| im::Vector::from_iter(q.tracks.into_iter().map(|t| t.url))),
					});
					Handled::Yes
				}
				_ if cmd.is(CLIENT_SUBMIT_PLAYLIST_REQ) => {
					if let State::Connected(data) = data {
						let token = data.token.0;
						let tracks = Vec::from_iter(data.tracklist.tracks.iter().cloned());
						self.request(ctx, |_handle, _id, mut client| async move {
							client
								.submit_playlist(pb::SubmitPlaylistRequest {
									token,
									playlist: Some(pb::Playlist {
										tracks: tracks
											.iter()
											.map(|t| pb::Track {
												url: t.source.as_str().to_owned(),
											})
											.collect(),
									}),
								})
								.await
								.unwrap();
						});
					}
					Handled::Yes
				}
				_ if cmd.is(CLIENT_JOIN_QUEUE_REQ) => {
					if let State::Connected(data) = data {
						let token = data.token.0;
						self.request(ctx, |handle, id, mut client| async move {
							client
								.join_queue(pb::JoinQueueRequest { token })
								.await
								.unwrap();
							handle
								.submit_command(CLIENT_JOIN_QUEUE_RES, (), id)
								.unwrap();
						});
					} else {
						error!("not yet connected")
					}
					Handled::Yes
				}
				_ if cmd.is(CLIENT_JOIN_QUEUE_RES) => {
					if let State::Connected(data) = data {
						data.in_queue = true;
					}
					Handled::Yes
				}
				_ if cmd.is(CLIENT_LEAVE_QUEUE_REQ) => {
					if let State::Connected(data) = data {
						let token = data.token.0;
						self.request(ctx, |handle, id, mut client| async move {
							client
								.join_queue(pb::JoinQueueRequest { token })
								.await
								.unwrap();
							handle
								.submit_command(CLIENT_JOIN_QUEUE_RES, (), id)
								.unwrap();
						});
					} else {
						error!("not yet cconnected")
					}
					Handled::Yes
				}
				_ if cmd.is(CLIENT_LEAVE_QUEUE_RES) => {
					if let State::Connected(data) = data {
						data.in_queue = false;
					}
					Handled::Yes
				}
				_ => Handled::No,
			},
			_ => Handled::No,
		};
		if handled.is_handled() {
			ctx.set_handled();
		}
		child.event(ctx, event, data, env);
	}

	fn lifecycle(
		&mut self,
		child: &mut W,
		ctx: &mut druid::LifeCycleCtx,
		event: &druid::LifeCycle,
		data: &State,
		env: &druid::Env,
	) {
		if let LifeCycle::WidgetAdded = event {
			self.spawn_playback_event_thread(ctx.get_external_handle());
		}
		child.lifecycle(ctx, event, data, env)
	}
}
