use std::{rc::Rc, time::Duration};

use druid::{
	widget::{Button, Container, Flex, Label, Maybe, Painter, SizedBox},
	BoxConstraints, Data, EventCtx, Lens, Size, Widget, WidgetExt,
};
use tf_db::Track;
use tf_player::player::state::Playing;

use super::{draw_icon_button, ICON_NEXT, ICON_PAUSE, ICON_PLAY, ICON_PREV};
use crate::{
	controller::playback,
	theme,
	widget::{overlay, player_bar::PlayerBar},
};

#[derive(Clone, Data, Lens)]
pub struct MediaBarState {
	pub playing: Rc<Playing>,
	pub current_track: Option<Rc<Track>>,
}

pub fn ui() -> impl Widget<MediaBarState> {
	let buttons = Flex::row()
		.with_child(prev_button())
		.with_default_spacer()
		.with_child(play_pause_button())
		.with_default_spacer()
		.with_child(next_button());

	let right_buttons = Flex::row()
		.with_flex_spacer(1.0)
		.with_child(Button::new("☰").on_click(
			move |ctx: &mut EventCtx, _: &mut MediaBarState, _| {
				ctx.submit_command(overlay::SHOW_MIDDLE.with((
					BoxConstraints::tight(Size::new(300.0, 300.0)),
					Box::new(move |env| {
						Box::new(
							Container::new(super::queue::ui())
								.border(env.get(theme::FOREGROUND), 1.0),
						)
					}),
				)))
			},
		));

	let track_info = Maybe::new(
		|| Label::new(|data: &Rc<Track>, _: &_| format!("{} - {}", data.artist, data.title)),
		|| SizedBox::empty(),
	)
	.expand_width();

	Flex::column()
		.with_child(
			Flex::row()
				.with_flex_child(track_info.lens(MediaBarState::current_track), 1.0)
				.with_child(buttons.lens(MediaBarState::playing))
				.with_flex_child(right_buttons, 1.0),
		)
		.with_child(
			Flex::row()
				.with_child(Label::new(|data: &Rc<Playing>, _: &_| {
					format_duration(&data.offset)
				}))
				.with_default_spacer()
				.with_flex_child(PlayerBar::default(), 1.0)
				.with_default_spacer()
				.with_child(Label::new(|data: &Rc<Playing>, _: &_| {
					format_duration(&data.track.duration)
				}))
				.lens(MediaBarState::playing),
		)
		.expand_width()
}

fn play_pause_button() -> impl Widget<Rc<Playing>> {
	Painter::new(|ctx, data: &Rc<Playing>, env| {
		draw_icon_button(ctx, env, if data.paused { ICON_PLAY } else { ICON_PAUSE })
	})
	.fix_size(36.0, 36.0)
	.on_click(|ctx: &mut EventCtx, _, _| {
		ctx.submit_command(playback::PLAYER_PLAY_PAUSE);
	})
}

fn prev_button() -> impl Widget<Rc<Playing>> {
	Painter::new(|ctx, _, env| draw_icon_button(ctx, env, ICON_PREV))
		.fix_size(36.0, 36.0)
		.on_click(|ctx: &mut EventCtx, _, _| {
			ctx.submit_command(playback::PLAYER_PREV);
		})
}

fn next_button() -> impl Widget<Rc<Playing>> {
	Painter::new(|ctx, _, env| draw_icon_button(ctx, env, ICON_NEXT))
		.fix_size(36.0, 36.0)
		.on_click(|ctx: &mut EventCtx, _, _| {
			ctx.submit_command(playback::PLAYER_NEXT);
		})
}

fn format_duration(d: &Duration) -> String {
	format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60)
}
