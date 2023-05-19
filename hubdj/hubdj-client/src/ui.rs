use std::{mem, rc::Rc};

use druid::{
	keyboard_types::Key,
	lens,
	widget::{Button, Flex, Label, List, Maybe, SizedBox, TextBox, ViewSwitcher},
	ArcStr, EventCtx, TextAlignment, Widget, WidgetExt,
};
use hubdj_core::UserId;
use tf_gui::{theme, widget::controllers::OnKey};

use crate::{
	controllers::{
		client::{
			ClientController, CLIENT_CONNECT_REQ, CLIENT_JOIN_QUEUE_REQ, CLIENT_LEAVE_QUEUE_REQ,
			CLIENT_SUBMIT_PLAYLIST_REQ,
		},
		query::{QueryController, QUERY_RUN},
	},
	state::{Booth, State, StateConnected, StateDisconnected, Track, Tracklist, User, UserState},
};

pub fn ui(db: &tf_db::Client, client_controller: ClientController) -> impl Widget<State> {
	let db = db.clone();
	ViewSwitcher::new(
		|data: &State, _| mem::discriminant(data),
		move |_, data, _| match data {
			State::Disconnected(_) => disconnected_ui()
				.lens(enum_lens!(State::Disconnected))
				.boxed(),
			State::Connected(_) => connected_ui(&db).lens(enum_lens!(State::Connected)).boxed(),
		},
	)
	.padding(16.0)
	.controller(client_controller)
}

fn disconnected_ui() -> impl Widget<StateDisconnected> {
	Flex::column()
		.with_child(
			TextBox::new()
				.with_placeholder("Your name")
				.lens(StateDisconnected::name),
		)
		.with_child(
			Button::new("Connect")
				.on_click(|ctx, _data, _env| ctx.submit_command(CLIENT_CONNECT_REQ)),
		)
}

fn connected_ui(db: &tf_db::Client) -> impl Widget<StateConnected> {
	Flex::column()
		.with_child(Label::new(|data: &StateConnected, _: &_| {
			format!("Connected as {}", data.name)
		}))
		.with_child(
			Maybe::new(
				|| {
					Flex::row()
						.with_child(
							Label::new(|data: &ArcStr, _: &_| format!("Track: {data}"))
								.lens(Booth::track),
						)
						.with_child(
							Label::new(|data: &Rc<UserId>, _: &_| format!("Current DJ: {data:?}"))
								.lens(Booth::dj),
						)
				},
				SizedBox::empty,
			)
			.lens(StateConnected::booth),
		)
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
		.with_default_spacer()
		.with_child(booth_controls())
		.with_default_spacer()
		.with_child(tracklist(&db.clone()).lens(StateConnected::tracklist))
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

fn booth_controls() -> impl Widget<StateConnected> {
	Button::new(|data: &StateConnected, _: &_| match data.in_queue {
		false => "Join the queue",
		true => "Joined!",
	})
	.on_click(
		|ctx: &mut EventCtx, data: &mut StateConnected, _| match data.in_queue {
			false => ctx.submit_command(CLIENT_JOIN_QUEUE_REQ),
			true => ctx.submit_command(CLIENT_LEAVE_QUEUE_REQ),
		},
	)
	.env_scope(|env, data: &StateConnected| match data.in_queue {
		false => {
			env.set(druid::theme::BUTTON_DARK, env.get(theme::THEME_BLUE));
			env.set(druid::theme::BUTTON_LIGHT, env.get(theme::THEME_BLUE));
		}
		true => {
			env.set(druid::theme::BUTTON_DARK, env.get(theme::THEME_GREEN));
			env.set(druid::theme::BUTTON_LIGHT, env.get(theme::THEME_GREEN));
		}
	})
}

fn tracklist(db: &tf_db::Client) -> impl Widget<Tracklist> {
	Flex::column()
		.with_child(
			TextBox::new()
				.with_placeholder("*")
				.with_text_alignment(TextAlignment::Center)
				.controller(OnKey::new(Key::Enter, |ctx, data: &mut String, _| {
					ctx.submit_notification(QUERY_RUN.with(data.to_owned()))
				}))
				.expand_width()
				.lens(Tracklist::query),
		)
		.with_child(
			Button::new("Add all")
				.on_click(|ctx, _, _| ctx.submit_command(CLIENT_SUBMIT_PLAYLIST_REQ)),
		)
		.with_child(
			List::new(|| {
				Flex::row().with_child(
					Label::new(|data: &ArcStr, _: &_| format!("Title: {data}")).lens(Track::title),
				)
			})
			.lens(Tracklist::tracks),
		)
		.controller(QueryController::new(db))
}
