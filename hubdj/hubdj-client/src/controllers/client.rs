use std::{future::Future, rc::Rc};

use anyhow::Result;
use druid::{
	im, widget::Controller, Event, ExtEventSink, Handled, Selector, SingleUse, Widget, WidgetId,
};
use hubdj_core::{UserId, UserToken};
use tonic::{transport::Channel, Response};

use crate::{
	pb::{self, hubdj_client::HubdjClient, AuthRequest, AuthResponse},
	state::{State, StateConnected, StateDisconnected, Tracklist, User, UserState},
};

pub const CLIENT_CONNECT_REQ: Selector = Selector::new("client.connect.req");
pub const CLIENT_CONNECT_RES: Selector<SingleUse<Response<AuthResponse>>> =
	Selector::new("client.connect.res");
pub const CLIENT_GET_USER_RES: Selector<SingleUse<Response<pb::User>>> =
	Selector::new("client.get-user.res");

pub struct ClientController {
	client: HubdjClient<Channel>,
}

impl ClientController {
	pub async fn new() -> Result<Self> {
		Ok(Self {
			client: HubdjClient::connect("http://[::1]:53000").await?,
		})
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
					});
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
				_ => Handled::No,
			},
			_ => Handled::No,
		};
		if handled.is_handled() {
			ctx.set_handled();
		}
		child.event(ctx, event, data, env);
	}
}
