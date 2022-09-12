use std::{rc::Rc, sync::Arc};

use druid::{
	im, lens,
	widget::{Container, EnvScope, Flex, Label, List, Painter, SizedBox},
	Color, Data, EventCtx, Key, Lens, LensExt, Widget, WidgetExt,
};
use tf_db::Track;
use uuid::Uuid;

use super::{draw_icon_button, ICON_DELETE, ICON_EDIT, ICON_PAUSE, ICON_PLAY};
use crate::{
	command,
	controller::playback::{PLAYER_CLEAR, PLAYER_ENQUEUE, PLAYER_PLAY_PAUSE},
	data::ctx::Ctx,
	theme,
	widget::{common::focusable_button::FocusableButton, controllers::AutoFocus, overlay},
	State,
};

const TRACK_LIST_ITEM_BACKGROUND: Key<Color> = Key::new("track_list.item.background");

#[derive(Clone, Data, Lens)]
pub struct TrackCtx {
	pub playing: Option<Rc<Uuid>>,
	pub selected: Option<Rc<Uuid>>,
}

pub fn ui() -> impl Widget<State> {
	let title_column = List::new(|| {
		Flex::column()
			.with_child(
				Label::new(|track: &Rc<Track>, _: &_| track.title.to_owned())
					.with_text_size(16.0)
					.fix_height(24.0),
			)
			.with_child(EnvScope::new(
				|env, _| env.set(druid::theme::TEXT_COLOR, env.get(theme::FOREGROUND_DIM)),
				Label::new(|item: &Rc<Track>, _: &_| item.artist.to_owned())
					.with_text_size(13.0)
					.fix_height(10.0),
			))
			.lens(Ctx::<TrackCtx, Rc<Track>>::data())
			.fix_height(64.0)
			.background(TRACK_LIST_ITEM_BACKGROUND)
			.env_scope(|env, state: &Ctx<_, _>| {
				env.set(
					TRACK_LIST_ITEM_BACKGROUND,
					if state.ctx.selected.as_deref() == Some(&state.data.id) {
						env.get(crate::theme::BACKGROUND_HIGHLIGHT0)
					} else {
						Color::TRANSPARENT
					},
				)
			})
	});

	let tag_columns = List::new(|| {
		Flex::column()
			.with_child(Label::new(|s: &Ctx<_, (String, f32)>, _: &_| {
				s.data.0.clone()
			}))
			.with_spacer(32.0)
			.with_child(
				List::new(|| {
					Label::new(|s: &Ctx<(String, f32), Rc<Track>>, _: &_| String::from("aaaa"))
				})
				.scroll()
				.lens(Ctx::make(Ctx::data(), Ctx::ctx())),
			)
	})
	.horizontal()
	.lens(State::tracks.then(Ctx::make(
		lens::Map::new(|s: &im::Vector<Rc<Track>>| s.clone(), |_, _| {}),
		lens::Map::new(
			|s: &im::Vector<Rc<Track>>| Arc::new(s[0].tags.clone()),
			|_, _| {},
		),
	)));

	Flex::row()
		.with_flex_child(
			Flex::column()
				.with_child(Label::new("title"))
				.with_spacer(32.0)
				.with_child(title_column)
				.expand_width()
				.border(Color::WHITE, 1.0)
				.lens(Ctx::make(
					lens::Map::new(
						|s: &State| TrackCtx {
							playing: s.current_track.as_ref().map(|t| Rc::new(t.id)),
							selected: s.selected_track.as_ref().cloned(),
						},
						|_, _| {},
					),
					State::tracks,
				)),
			1.0,
		)
		.with_child(tag_columns)

	// table.expand_width().lens(Ctx::make(
	// 	lens::Map::new(
	// 		|s: &State| TrackCtx {
	// 			playing: s.current_track.as_ref().map(|t| Rc::new(t.id)),
	// 			selected: s.selected_track.as_ref().cloned(),
	// 		},
	// 		|_, _| {},
	// 	),
	// 	State::tracks,
	// ))
}

// pub fn ui() -> impl Widget<State> {
// 	List::new(song_ui).expand_width().lens(Ctx::make(
// 		lens::Map::new(
// 			|s: &State| TrackCtx {
// 				playing: s.current_track.as_ref().map(|t| Rc::new(t.id)),
// 				selected: s.selected_track.as_ref().cloned(),
// 			},
// 			|_, _| {},
// 		),
// 		State::tracks,
// 	))
// }

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
					let track_id = track.id;
					ctx.submit_command(overlay::SHOW_MODAL.with((
						Color::rgba(1.0, 1.0, 1.0, 0.1),
						Box::new(move |_| {
							Container::new(
								Flex::column()
									.with_child(Label::new("Delete this track?"))
									.with_default_spacer()
									.with_child(
										Flex::row()
											.with_child(
												FocusableButton::new("Cancel")
													.on_click(move |ctx, _, _| {
														ctx.submit_command(overlay::HIDE);
													})
													.controller(AutoFocus),
											)
											.with_default_spacer()
											.with_child(
												FocusableButton::new("Delete")
													.on_click(move |ctx, _, _| {
														ctx.submit_command(
															command::TRACK_DELETE.with(track_id),
														);
														ctx.submit_command(overlay::HIDE);
													})
													.env_scope(|env, _| {
														env.set(
															druid::theme::BUTTON_DARK,
															Color::RED,
														)
													}),
											),
									),
							)
							.padding(8.0)
							.background(theme::BACKGROUND)
							.boxed()
						}),
					)));
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
			if data.ctx.playing.as_deref() == Some(&data.data.id) {
				ctx.submit_command(PLAYER_PLAY_PAUSE);
			} else {
				ctx.submit_command(PLAYER_CLEAR);
				ctx.submit_command(PLAYER_ENQUEUE.with(data.data.clone()));
			}
		},
	)
}
