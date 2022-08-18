use std::rc::Rc;

use druid::{
	im, lens,
	widget::{Container, EnvScope, Flex, Label, List, Painter, SizedBox},
	Color, Data, EventCtx, Lens, Widget, WidgetExt,
};
use tf_db::Track;
use uuid::Uuid;

use super::{
	draw_icon_button, ICON_DELETE, ICON_EDIT, ICON_PAUSE, ICON_PLAY, TRACK_LIST_ITEM_BACKGROUND,
};
use crate::{command, data::ctx::Ctx, theme, State};

#[derive(Clone, Data, Lens)]
pub struct TrackCtx {
	pub playing: Option<Rc<Uuid>>,
	pub selected: Option<Rc<Uuid>>,
}

pub fn ui() -> impl Widget<State> {
	List::new(song_ui).expand_width().lens(Ctx::make(
		lens::Map::new(
			|s: &State| TrackCtx {
				playing: s.current_track.as_ref().map(|t| Rc::new(t.id)),
				selected: s.selected_track.as_ref().cloned(),
			},
			|_, _| {},
		),
		State::tracks,
	))
}

fn song_ui() -> impl Widget<Ctx<TrackCtx, Rc<Track>>> {
	let row = Flex::row()
		.with_child(play_track_button())
		.with_default_spacer()
		.with_flex_child(
			Flex::column()
				.with_child(
					Label::new(|track: &Rc<Track>, _: &_| track.title.to_owned())
						.with_text_size(16.0)
						.fix_height(24.0)
						.expand_width(),
				)
				.with_child(EnvScope::new(
					|env, _| env.set(druid::theme::TEXT_COLOR, env.get(theme::FOREGROUND_DIM)),
					Label::new(|item: &Rc<Track>, _: &_| item.artist.to_owned())
						.with_text_size(13.0)
						.fix_height(10.0)
						.expand_width(),
				))
				.lens(Ctx::<TrackCtx, Rc<Track>>::data()),
			1.0,
		)
		.with_child(
			Painter::new(|ctx, _, env| draw_icon_button(ctx, env, ICON_EDIT))
				.fix_size(36.0, 36.0)
				.on_click(|ctx: &mut EventCtx, track: &mut Rc<Track>, _| {
					ctx.submit_command(command::UI_TRACK_EDIT_OPEN.with(track.id))
				})
				.lens(Ctx::<TrackCtx, Rc<Track>>::data()),
		)
		.with_child(
			Painter::new(|ctx, _, env| draw_icon_button(ctx, env, ICON_DELETE))
				.fix_size(36.0, 36.0)
				.on_click(|ctx: &mut EventCtx, track: &mut Rc<Track>, _| {
					ctx.submit_command(command::TRACK_DELETE.with(track.id))
				})
				.lens(Ctx::<TrackCtx, Rc<Track>>::data()),
		)
		.expand_width()
		.fix_height(64.0);

	EnvScope::new(
		|env, state: &Ctx<_, _>| {
			env.set(
				TRACK_LIST_ITEM_BACKGROUND,
				if state.ctx.selected.as_deref() == Some(&state.data.id) {
					env.get(crate::theme::BACKGROUND_HIGHLIGHT0)
				} else {
					Color::TRANSPARENT
				},
			)
		},
		Container::new(row).background(TRACK_LIST_ITEM_BACKGROUND),
	)
}

fn play_track_button() -> impl Widget<Ctx<TrackCtx, Rc<Track>>> {
	Painter::new(|ctx, data: &Ctx<TrackCtx, Rc<Track>>, env| {
		draw_icon_button(
			ctx,
			env,
			if data.ctx.playing.as_deref() == Some(&data.data.id) {
				ICON_PAUSE
			} else {
				ICON_PLAY
			},
		)
	})
	.fix_size(36.0, 36.0)
	.on_click(
		|ctx: &mut EventCtx, data: &mut Ctx<TrackCtx, Rc<Track>>, _| {
			ctx.submit_command(command::TRACK_PLAY.with(data.data.id));
		},
	)
}
