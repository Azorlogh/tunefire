use std::rc::Rc;

use druid::{
	keyboard_types::Key,
	kurbo::{BezPath, Circle},
	lens::Map,
	widget::{ControllerHost, Flex, Label, Maybe, Painter, Scroll, SizedBox, TextBox},
	Affine, Env, EventCtx, PaintCtx, RenderContext, TextAlignment, Vec2, Widget, WidgetExt,
};
use tf_player::player;

use self::media_bar::MediaBarState;
use crate::{
	command,
	controller::{playback::PlaybackController, search::SearchController},
	data::ctx::Ctx,
	theme,
	widget::{common::stack::Stack, controllers::OnKey, overlay::Overlay, search_bar::SearchBar},
	State,
};

mod add_track;
mod media_bar;
mod queue;
mod track_edit;
mod track_list;

pub fn ui() -> impl Widget<State> {
	let query_box = query_box();

	let main_view = Flex::row()
		.with_flex_child(
			Scroll::new(track_list::ui()).vertical().expand_height(),
			1.0,
		)
		.with_child(Maybe::new(|| track_edit::ui(), || SizedBox::empty()).lens(State::track_edit));

	let mut root = Flex::column();
	root.add_default_spacer();
	root.add_child(query_box);
	root.add_default_spacer();
	root.add_flex_child(main_view, 1.0);
	root.add_default_spacer();
	root.add_child(search_bar());
	root.add_default_spacer();
	root.add_child(
		Maybe::new(|| media_bar::ui(), || SizedBox::empty()).lens(Map::new(
			|s: &State| {
				s.player_state.get_playing().map(|p| MediaBarState {
					playing: Rc::new(p.clone()),
					current_track: s.current_track.clone(),
					volume: s.volume,
				})
			},
			|s: &mut State, inner: Option<MediaBarState>| {
				if let Some(inner) = inner {
					s.volume = inner.volume;
					s.player_state =
						Rc::new(player::State::Playing((*inner.playing).clone()).clone());
				}
			},
		)),
	);

	Stack::new()
		.with_child(
			root.padding(10.0)
				.expand_width()
				.controller(PlaybackController::new().expect("Couldn't create playback controller"))
				.controller(SearchController::new().expect("Couldn't create plugin controller")),
		)
		.with_child(
			Maybe::new(|| add_track::add_track(), || SizedBox::empty()).lens(State::new_track),
		)
		.with_child(Overlay::new())
}

fn query_box() -> impl Widget<State> {
	Flex::row()
		.with_child(play_query_button())
		.with_default_spacer()
		.with_flex_child(
			ControllerHost::new(
				TextBox::new()
					.with_placeholder("*")
					.with_text_alignment(TextAlignment::Center),
				OnKey::new(Key::Enter, |ctx, _, _| {
					ctx.submit_command(command::QUERY_RUN)
				}),
			)
			.expand_width()
			.lens(State::query),
			1.0,
		)
		.with_default_spacer()
		.with_child(
			Label::new("TUNEFIRE")
				.with_font(druid::theme::UI_FONT_BOLD)
				.with_text_color(theme::FOREGROUND_DIM),
		)
		.with_default_spacer()
}

fn search_bar() -> impl Widget<State> {
	SearchBar::new().lens(Ctx::make(
		State::track_search_results,
		State::new_track_search,
	))
}

fn play_query_button() -> impl Widget<State> {
	Painter::new(|ctx, _: &State, env| draw_icon_button(ctx, env, ICON_FIRE))
		.fix_size(36.0, 36.0)
		.on_click(|ctx: &mut EventCtx, _, _| {
			ctx.submit_command(command::QUERY_PLAY);
		})
}

pub const ICON_FIRE: &str = include_str!("../../assets/fire.svg");
pub const ICON_PLAY: &str = include_str!("../../assets/play.svg");
pub const ICON_PAUSE: &str = include_str!("../../assets/pause.svg");
pub const ICON_PREV: &str = include_str!("../../assets/previous.svg");
pub const ICON_NEXT: &str = include_str!("../../assets/next.svg");
pub const ICON_EDIT: &str = include_str!("../../assets/edit.svg");
pub const ICON_DELETE: &str = include_str!("../../assets/delete.svg");
pub const _ICON_SETTINGS: &str = include_str!("../../assets/settings.svg");

pub fn draw_icon_button(ctx: &mut PaintCtx, env: &Env, icon_svg: &str) {
	let size = ctx.size();
	let rad = size.min_side() / 2.0;
	if ctx.is_hot() && ctx.has_focus() {
		ctx.fill(
			Circle::new((size.to_vec2() / 2.0).to_point(), rad),
			&env.get(crate::theme::BACKGROUND_HIGHLIGHT1),
		);
	}
	// let rad = size.min_side() / 48.0 * 0.75;
	let color = env.get(if ctx.is_hot() {
		crate::theme::FOREGROUND
	} else {
		crate::theme::FOREGROUND_DIM
	});
	ctx.fill(
		Affine::translate(Vec2::new(size.min_side() / 2.0, size.min_side() / 2.0))
			* Affine::scale(size.min_side() / 2.0 * 0.75)
			* Affine::scale(2.0 / 48.0)
			* Affine::translate(Vec2::new(-24.0, -24.0))
			* BezPath::from_svg(icon_svg).unwrap(),
		&color,
	);
}
