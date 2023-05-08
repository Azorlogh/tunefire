mod controllers;
mod theme;
#[macro_use]
mod util;
use std::{mem, rc::Rc};

use controllers::client::{ClientController, CLIENT_CONNECT_REQ};
use druid::{
	im, lens,
	widget::{Button, Flex, Label, List, Maybe, TextBox, ViewSwitcher},
	AppLauncher, Data, Lens, Widget, WidgetExt, WindowDesc,
};
use hubdj_core::{UserId, UserToken};

pub mod pb {
	tonic::include_proto!("hubdj");
}

// mod client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let controller = ClientController::new().await?;

	let state = State::Disconnected(StateDisconnected {
		name: String::from("Bob"),
	});

	AppLauncher::with_window(WindowDesc::new(ui(controller)))
		.log_to_console()
		.configure_env(theme::apply)
		.launch(state)
		.unwrap();

	Ok(())
}

#[derive(Clone, Data)]
pub enum State {
	Disconnected(StateDisconnected),
	Connected(StateConnected),
}

#[derive(Clone, Data, Lens)]
pub struct StateDisconnected {
	name: String,
}

#[derive(Clone, Data, Lens)]
pub struct StateConnected {
	id: Rc<UserId>,
	token: Rc<UserToken>,
	name: String,
	booth: Option<Booth>,
	users: im::OrdMap<Rc<UserId>, UserState>,
}

#[derive(Clone, Data, Lens)]
pub struct Booth {
	dj: Rc<UserId>,
	song: Song,
}

#[derive(Clone, Data, Lens)]
pub struct Song {
	url: String,
	artist: String,
	title: String,
}

#[derive(Clone, Data)]
pub enum UserState {
	Loading,
	Loaded(User),
}

#[derive(Clone, Data, Lens)]
pub struct User {
	id: Rc<UserId>,
	name: String,
	queue: Option<im::Vector<String>>,
}

fn ui(client_controller: ClientController) -> impl Widget<State> {
	ViewSwitcher::new(
		|data: &State, _| mem::discriminant(data),
		|_, data, _| match data {
			State::Disconnected(_) => disconnected_ui()
				.lens(enum_lens!(State::Disconnected))
				.boxed(),
			State::Connected(_) => connected_ui().lens(enum_lens!(State::Connected)).boxed(),
		},
	)
	.padding(16.0)
	.controller(client_controller)
}

fn disconnected_ui() -> impl Widget<StateDisconnected> {
	Flex::column()
		.with_child(TextBox::new().lens(StateDisconnected::name))
		.with_child(
			Button::new("Connect")
				.on_click(|ctx, _data, _env| ctx.submit_command(CLIENT_CONNECT_REQ)),
		)
}

fn connected_ui() -> impl Widget<StateConnected> {
	Flex::column()
		.with_child(Label::new(|data: &StateConnected, _: &_| {
			format!("Connected as {}", data.name)
		}))
		.with_spacer(16.0)
		.with_flex_child(
			List::new(|| {
				ViewSwitcher::new(
					|data, _| mem::discriminant(data),
					|_, data, _| match data {
						UserState::Loading => Label::new("Loading...").boxed(),
						UserState::Loaded(_) => {
							user_ui().lens(enum_lens!(UserState::Loaded)).boxed()
						}
					},
				)
			})
			.horizontal()
			.lens(StateConnected::users),
			1.0,
		)
}

fn user_ui() -> impl Widget<User> {
	Flex::column()
		.with_child(Label::new(|data: &User, _: &_| format!("👤 {}", data.name)))
		.with_child(
			Maybe::new(
				|| List::new(|| Label::new("track")),
				|| Label::new("Just listening..."),
			)
			.lens(User::queue),
		)
		.fix_width(192.0)
		.expand_height()
		.border(theme::FOREGROUND_DIM, 1.0)
}
