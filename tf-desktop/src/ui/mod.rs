use std::rc::Rc;

use druid::{
	im,
	kurbo::{BezPath, Circle},
	lens::Map,
	theme,
	widget::{
		Button, Container, ControllerHost, CrossAxisAlignment, EnvScope, Flex, Label, List, Maybe,
		Painter, Scroll, SizedBox, TextBox,
	},
	Affine, Color, Env, EventCtx, Insets, Key, PaintCtx, RenderContext, TextAlignment, Widget,
	WidgetExt,
};
use tf_player::player;

use crate::{
	command,
	state::{NewSong, SongEdit, SongListItem},
	widget::{
		common::stack::Stack, controllers::Enter, overlay::Overlay, player_tick::PlayerTick,
		tag_edit::TagEdit,
	},
	State,
};

use self::media_bar::MediaBarState;

const SONG_LIST_ITEM_BACKGROUND: Key<Color> = Key::new("song_list.item.background");

mod media_bar;

pub fn ui() -> impl Widget<State> {
	let query_box = query_box();

	let main_view = Flex::row()
		.with_flex_child(
			Scroll::new(songs_ui().lens(State::songs))
				.vertical()
				.expand_height(),
			1.0,
		)
		.with_child(Maybe::new(|| song_edit(), || SizedBox::empty()).lens(State::song_edit));

	let mut root = Flex::column();
	root.add_child(Label::new("T𝕌NEF𝕀RE"));
	root.add_default_spacer();
	root.add_child(query_box);
	root.add_default_spacer();
	root.add_flex_child(main_view, 1.0);
	root.add_default_spacer();
	root.add_child(add_song_ui());
	root.add_default_spacer();
	root.add_child(
		Maybe::new(|| media_bar::ui(), || SizedBox::empty()).lens(Map::new(
			|s: &State| {
				s.player_state.get_playing().map(|p| MediaBarState {
					playing: Rc::new(p.clone()),
					current_song: s.current_song.clone(),
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
			.with_child(Overlay::new()),
		PlayerTick::default(),
	)
}

fn songs_ui() -> impl Widget<im::Vector<SongListItem>> {
	List::new(|| {
		let row = Flex::row()
			.with_child(play_song_button())
			.with_flex_child(
				Label::new(|item: &SongListItem, _: &_| item.song.title.to_owned())
					.padding(Insets::uniform(10.0))
					.expand_width(),
				1.0,
			)
			.with_child(
				Painter::new(|ctx, _, env| draw_icon_button(ctx, env, ICON_EDIT))
					.fix_size(36.0, 36.0)
					.on_click(|ctx: &mut EventCtx, item: &mut SongListItem, _| {
						ctx.submit_command(command::UI_SONG_EDIT_OPEN.with(item.song.id))
					}),
			)
			.with_child(
				Painter::new(|ctx, _, env| draw_icon_button(ctx, env, ICON_DELETE))
					.fix_size(36.0, 36.0)
					.on_click(|ctx: &mut EventCtx, item: &mut SongListItem, _| {
						ctx.submit_command(command::SONG_DELETE.with(item.song.id))
					}),
			)
			.expand_width()
			.fix_height(48.0);

		EnvScope::new(
			|env, state| {
				env.set(
					SONG_LIST_ITEM_BACKGROUND,
					if state.selected {
						env.get(crate::theme::BACKGROUND_HIGHLIGHT0)
					} else {
						Color::TRANSPARENT
					},
				)
			},
			Container::new(row).background(SONG_LIST_ITEM_BACKGROUND),
		)
	})
	.expand_width()
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
}

fn song_edit() -> impl Widget<SongEdit> {
	let col =
		Flex::column()
			.cross_axis_alignment(CrossAxisAlignment::Fill)
			.with_child(
				Flex::row()
					.with_child(Label::new("title"))
					.with_child(TextBox::new().lens(SongEdit::title)),
			)
			.with_default_spacer()
			.with_child(
				Flex::row()
					.with_child(Label::new("source"))
					.with_child(TextBox::new().lens(SongEdit::source)),
			)
			.with_default_spacer()
			.with_child(List::new(|| TagEdit::new()).lens(SongEdit::tags))
			.with_child(Button::new("+").on_click(|ctx, data: &mut SongEdit, _| {
				ctx.submit_command(command::TAG_ADD.with(*data.id))
			}))
			.with_flex_spacer(1.0)
			.with_child(Button::new("CLOSE").on_click(|ctx, _: &mut SongEdit, _| {
				ctx.submit_command(command::UI_SONG_EDIT_CLOSE)
			}))
			.env_scope(|env, _| env.set(theme::BORDER_DARK, Color::TRANSPARENT))
			.fix_width(400.0)
			.padding(8.0);
	Container::new(col).background(crate::theme::BACKGROUND_HIGHLIGHT0)
}

fn add_song_ui() -> impl Widget<State> {
	Flex::row()
		.with_flex_child(
			Flex::column()
				.with_child(
					TextBox::new()
						.with_placeholder("Title")
						.lens(NewSong::title)
						.lens(State::new_song)
						.expand_width(),
				)
				.with_child(
					TextBox::new()
						.with_placeholder("Source")
						.lens(NewSong::source)
						.lens(State::new_song)
						.expand_width(),
				)
				.expand_width(),
			1.0,
		)
		.with_child(Button::new("+").on_click(|ctx, state: &mut State, _| {
			ctx.submit_command(command::SONG_ADD.with(state.new_song.clone()))
		}))
		.cross_axis_alignment(CrossAxisAlignment::Fill)
}

fn play_query_button() -> impl Widget<State> {
	Painter::new(|ctx, _: &State, env| draw_icon_button(ctx, env, ICON_PLAY))
		.fix_size(36.0, 36.0)
		.on_click(|ctx: &mut EventCtx, _, _| {
			ctx.submit_command(command::QUERY_PLAY);
		})
}

fn play_song_button() -> impl Widget<SongListItem> {
	Painter::new(|ctx, _: &SongListItem, env| draw_icon_button(ctx, env, ICON_PLAY))
		.fix_size(36.0, 36.0)
		.on_click(|ctx: &mut EventCtx, item: &mut SongListItem, _| {
			ctx.submit_command(command::SONG_PLAY.with(item.song.id));
		})
}

pub const ICON_PLAY: &str = "M0.750,0.567 L0.750,1.433 L1.500,1.000 L0.750,0.567";
pub const ICON_PAUSE: &str = "M0.598,0.521 L0.598,1.479 L0.866,1.479 L0.866,0.521 L0.598,0.521 M1.402,0.521 L1.134,0.521 L1.134,1.479 L1.402,1.479 L1.402,0.521";
pub const ICON_PREV: &str = "M1.250,0.567 L0.700,0.885 L0.700,0.567 L0.500,0.567 L0.500,1.000 L0.500,1.433 L0.700,1.433 L0.700,1.115 L1.250,1.433 L1.250,0.567";
pub const ICON_NEXT: &str = "M0.750,1.433 L1.300,1.115 L1.300,1.433 L1.500,1.433 L1.500,1.000 L1.500,0.567 L1.300,0.567 L1.300,0.885 L0.750,0.567 L0.750,1.433";
pub const ICON_EDIT: &str = "M1.000 0.513 L0.700,0.513 L0.700,1.000 L0.700,1.487 L1.000,1.487 L1.300,1.487 L1.300,1.017 L1.024,1.017 L1.024,0.699 L1.210,0.513 L1.000,0.513 M1.654 0.564 L1.477,0.387 L1.124,0.741 L1.124,0.917 L1.300,0.917 L1.654,0.564";
pub const ICON_DELETE: &str = "M0.814 1.394 L1.000,1.394 L1.186,1.394 A0.044,0.044 0 0,0 1.229,1.357 L1.348,0.695 A0.044,0.044 0 0,0 1.393,0.651 A0.044,0.044 0 0,0 1.348,0.606 L0.652,0.606 A0.044,0.044 0 0,0 0.607,0.651 A0.044,0.044 0 0,0 0.652,0.695 L0.771,1.357 A0.044,0.044 0 0,0 0.814,1.394";

pub fn draw_icon_button(ctx: &mut PaintCtx, env: &Env, icon_svg: &str) {
	let size = ctx.size();
	let rad = size.min_side() / 2.0;
	if ctx.is_hot() {
		ctx.fill(
			Circle::new((size.to_vec2() / 2.0).to_point(), rad),
			&env.get(crate::theme::BACKGROUND_HIGHLIGHT1),
		);
	}
	ctx.fill(
		Affine::scale(rad) * BezPath::from_svg(icon_svg).unwrap(),
		&env.get(theme::FOREGROUND_LIGHT),
	);
}
