use std::rc::Rc;

use druid::{
	im,
	kurbo::{BezPath, Circle},
	lens::Map,
	widget::{
		Button, Container, ControllerHost, CrossAxisAlignment, EnvScope, Flex, Label, List, Maybe,
		Painter, Scroll, SizedBox, TextBox,
	},
	Affine, Color, Env, EventCtx, Key, PaintCtx, RenderContext, TextAlignment, Vec2, Widget,
	WidgetExt,
};
use tf_player::player;

use self::media_bar::MediaBarState;
use crate::{
	command,
	state::{TrackEdit, TrackListItem},
	theme,
	widget::{
		common::stack::Stack, controllers::Enter, overlay::Overlay, player_tick::PlayerTick,
		tag_edit::TagEdit,
	},
	State,
};

const TRACK_LIST_ITEM_BACKGROUND: Key<Color> = Key::new("track_list.item.background");

mod add_track;
mod media_bar;
mod track_list;

pub fn ui() -> impl Widget<State> {
	let query_box = query_box();

	let main_view = Flex::row()
		.with_flex_child(
			Scroll::new(track_list::ui()).vertical().expand_height(),
			1.0,
		)
		.with_child(Maybe::new(|| track_edit(), || SizedBox::empty()).lens(State::track_edit));

	let mut root = Flex::column();
	root.add_default_spacer();
	root.add_child(query_box);
	root.add_default_spacer();
	root.add_flex_child(main_view, 1.0);
	root.add_default_spacer();
	root.add_child(url_bar());
	root.add_default_spacer();
	root.add_child(
		Maybe::new(|| media_bar::ui(), || SizedBox::empty()).lens(Map::new(
			|s: &State| {
				s.player_state.get_playing().map(|p| MediaBarState {
					playing: Rc::new(p.clone()),
					current_track: s.current_track.clone(),
				})
			},
			|s: &mut State, inner: Option<MediaBarState>| {
				s.player_state = Rc::new(
					inner
						.map(|s| player::State::Playing((*s.playing).clone()))
						.unwrap_or(player::State::Idle),
				);
			},
		)),
	);
	ControllerHost::new(
		Stack::new()
			.with_child(root.padding(10.0).expand_width())
			.with_child(
				Maybe::new(|| add_track::add_track(), || SizedBox::empty()).lens(State::new_track),
			)
			.with_child(Overlay::new()),
		PlayerTick::default(),
	)
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
				Enter::new(|ctx, _, _| ctx.submit_command(command::QUERY_RUN)),
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

fn track_edit() -> impl Widget<TrackEdit> {
	let col =
		Flex::column()
			.cross_axis_alignment(CrossAxisAlignment::Fill)
			.with_child(
				Flex::row()
					.with_child(Label::new("title"))
					.with_child(TextBox::new().lens(TrackEdit::title)),
			)
			.with_default_spacer()
			.with_child(
				Flex::row()
					.with_child(Label::new("source"))
					.with_child(TextBox::new().lens(TrackEdit::source)),
			)
			.with_default_spacer()
			.with_child(List::new(|| TagEdit::new()).lens(TrackEdit::tags))
			.with_child(Button::new("+").on_click(|ctx, data: &mut TrackEdit, _| {
				ctx.submit_command(command::TAG_ADD.with(*data.id))
			}))
			.with_flex_spacer(1.0)
			.with_child(Button::new("CLOSE").on_click(|ctx, _: &mut TrackEdit, _| {
				ctx.submit_command(command::UI_TRACK_EDIT_CLOSE)
			}))
			.env_scope(|env, _| env.set(druid::theme::BORDER_DARK, Color::TRANSPARENT))
			.fix_width(400.0)
			.padding(8.0);
	Container::new(col).background(crate::theme::BACKGROUND_HIGHLIGHT0)
}

fn url_bar() -> impl Widget<State> {
	ControllerHost::new(
		TextBox::new().with_placeholder("Source"),
		Enter::new(|ctx, data: &mut String, _| {
			ctx.submit_command(command::UI_TRACK_ADD_OPEN.with(data.to_owned()))
		}),
	)
	.expand_width()
	.lens(State::new_track_url)
}

fn play_query_button() -> impl Widget<State> {
	Painter::new(|ctx, _: &State, env| draw_icon_button(ctx, env, ICON_PLAY))
		.fix_size(36.0, 36.0)
		.on_click(|ctx: &mut EventCtx, _, _| {
			ctx.submit_command(command::QUERY_PLAY);
		})
}

pub const ICON_PLAY: &str = include_str!("../../assets/play.svg");
pub const ICON_PAUSE: &str = include_str!("../../assets/pause.svg");
pub const ICON_PREV: &str = include_str!("../../assets/previous.svg");
pub const ICON_NEXT: &str = include_str!("../../assets/next.svg");
pub const ICON_EDIT: &str = include_str!("../../assets/edit.svg");
pub const ICON_DELETE: &str = include_str!("../../assets/delete.svg");

pub fn draw_icon_button(ctx: &mut PaintCtx, env: &Env, icon_svg: &str) {
	let size = ctx.size();
	let rad = size.min_side() / 2.0;
	if ctx.is_hot() {
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
