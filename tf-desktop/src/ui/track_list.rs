use std::{cell::RefCell, rc::Rc};

use druid::{
	lens,
	widget::{
		Axis, Container, CrossAxisAlignment, Either, EnvScope, Flex, Label, List,
		MainAxisAlignment, Painter, SizedBox,
	},
	Color, Data, EventCtx, Key, Lens, RenderContext, Widget, WidgetExt,
};
use tf_db::Track;
use uuid::Uuid;

use super::{draw_icon_button, ICON_DELETE, ICON_EDIT, ICON_PAUSE, ICON_PLAY};
use crate::{
	command,
	controller::playback::{PLAYER_CLEAR, PLAYER_ENQUEUE, PLAYER_PLAY_PAUSE},
	data::ctx::Ctx,
	theme,
	widget::{
		common::{
			focusable_button::FocusableButton, knob::Knob, separator::Separator, stack::Stack,
		},
		controllers::{AutoFocus, OnHotChange},
		overlay,
	},
	State,
};

const TRACK_LIST_ITEM_BACKGROUND: Key<Color> = Key::new("track_list.item.background");

#[derive(Clone, Data, Lens)]
pub struct TrackCtx {
	pub playing: Option<Rc<Uuid>>,
	pub selected: Option<Rc<Uuid>>,
}

const TRACK_HEIGHT: f64 = 64.0;

pub fn ui() -> impl Widget<State> {
	let track_title = || {
		Flex::column()
			.with_child(
				Label::new(|track: &Rc<RefCell<Track>>, _: &_| track.borrow().title.to_owned())
					.with_text_size(16.0)
					.fix_height(24.0),
			)
			.with_child(EnvScope::new(
				|env, _| env.set(druid::theme::TEXT_COLOR, env.get(theme::FOREGROUND_DIM)),
				Label::new(|item: &Rc<RefCell<Track>>, _: &_| item.borrow().artist.to_owned())
					.with_text_size(13.0)
					.fix_height(10.0),
			))
			.main_axis_alignment(MainAxisAlignment::Center)
			.cross_axis_alignment(CrossAxisAlignment::Start)
			.expand_width()
			.fix_height(64.0)
			.lens(Ctx::data())
	};

	let tag_columns = List::new(|| {
		Flex::row()
			.with_child(
				Separator::new()
					.with_axis(Axis::Vertical)
					.with_width(1.0)
					.with_color(theme::BACKGROUND_HIGHLIGHT1)
					.fix_height(0.0),
			)
			.with_child(
				Flex::column()
					.with_child(
						Flex::row()
							.with_default_spacer()
							.with_child(Label::new(|s: &Ctx<_, String>, _: &_| s.data.clone()))
							.with_default_spacer(),
					)
					.with_default_spacer()
					.with_child(
						List::new(|| {
							Flex::column()
								.with_child(
									Separator::new()
										.with_width(1.0)
										.with_color(theme::BACKGROUND_HIGHLIGHT1)
										.fix_width(0.0),
								)
								.with_child(
									Knob::new()
										.lens(lens::Map::new(
											|s: &Ctx<String, Rc<RefCell<Track>>>| {
												*s.data.borrow().tags.get(&s.ctx).unwrap_or(&0.0)
											},
											|s: &mut Ctx<String, Rc<RefCell<Track>>>,
											 inner: f32| {
												s.data
													.borrow_mut()
													.tags
													.insert(s.ctx.clone(), inner);
											},
										))
										.fix_width(TRACK_HEIGHT)
										.fix_height(TRACK_HEIGHT)
										.center(),
								)
								.cross_axis_alignment(CrossAxisAlignment::Fill)
						})
						.lens(Ctx::make(Ctx::data(), Ctx::ctx())),
					)
					.cross_axis_alignment(CrossAxisAlignment::Fill),
			)
			.cross_axis_alignment(CrossAxisAlignment::Fill)
	})
	.horizontal()
	.lens(Ctx::make(State::tracks, State::shown_tags));

	let table = Flex::row()
		.with_child(column_ui("", || play_track_button().center()).fix_width(64.0))
		.with_flex_child(column_ui("Title", track_title), 1.0)
		.with_child(Either::new(
			|s: &State, _: _| s.track_edit.is_none(),
			tag_columns,
			SizedBox::empty(),
		))
		.with_child(
			column_ui("", || {
				Painter::new(|ctx, _, env| draw_icon_button(ctx, env, ICON_EDIT))
					.fix_size(36.0, 36.0)
					.on_click(
						|ctx: &mut EventCtx, track: &mut Ctx<_, Rc<RefCell<Track>>>, _| {
							ctx.submit_command(
								command::UI_TRACK_EDIT_OPEN.with(track.data.borrow().id),
							)
						},
					)
					.center()
			})
			.fix_width(64.0),
		)
		.with_child(
			column_ui("", || {
				Painter::new(|ctx, _, env| draw_icon_button(ctx, env, ICON_DELETE))
					.fix_size(36.0, 36.0)
					.on_click(
						|ctx: &mut EventCtx, track: &mut Ctx<_, Rc<RefCell<Track>>>, _| {
							let track_id = track.data.borrow().id;
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
																	command::TRACK_DELETE
																		.with(track_id),
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
						},
					)
					.center()
			})
			.fix_width(64.0),
		)
		.with_default_spacer();

	Stack::new()
		.with_child(
			Flex::column()
				.with_child(Label::new(""))
				.with_default_spacer()
				.with_flex_child(
					List::new(|| {
						Painter::new(|ctx, _, env| {
							if ctx.is_hot() {
								let size = ctx.size().to_rect();
								ctx.fill(size, &env.get(theme::BACKGROUND_HIGHLIGHT0))
							}
						})
						.controller(OnHotChange::new(|ctx, _, _, _| ctx.request_paint()))
						.background(TRACK_LIST_ITEM_BACKGROUND)
						.env_scope(|env, state: &Ctx<TrackCtx, Rc<RefCell<Track>>>| {
							env.set(
								TRACK_LIST_ITEM_BACKGROUND,
								if state.ctx.selected.as_deref() == Some(&state.data.borrow().id) {
									env.get(crate::theme::BACKGROUND_HIGHLIGHT0)
								} else {
									Color::TRANSPARENT
								},
							)
						})
						.fix_height(TRACK_HEIGHT + 1.0)
					})
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
				.fix_height(0.0),
		)
		.with_child(table)
		.fix_height(0.0)
}

fn column_ui<W>(name: &str, inner: impl Fn() -> W + 'static) -> impl Widget<State>
where
	W: Widget<Ctx<TrackCtx, Rc<RefCell<Track>>>> + 'static,
{
	Flex::column()
		.with_child(Label::new(name).align_left())
		.with_default_spacer()
		.with_child(List::new(move || {
			Flex::column()
				.with_child(
					Separator::new()
						.with_width(1.0)
						.with_color(theme::BACKGROUND_HIGHLIGHT1),
				)
				.with_flex_child(inner(), 1.0)
				.background(TRACK_LIST_ITEM_BACKGROUND)
				.env_scope(|env, state: &_| {
					env.set(
						TRACK_LIST_ITEM_BACKGROUND,
						if state.ctx.selected.as_deref() == Some(&state.data.borrow().id) {
							env.get(crate::theme::BACKGROUND_HIGHLIGHT0)
						} else {
							Color::TRANSPARENT
						},
					)
				})
				.fix_height(TRACK_HEIGHT + 1.0)
		}))
		.lens(Ctx::make(
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

// fn song_ui() -> impl Widget<Ctx<TrackCtx, Rc<Track>>> {
// 	let row = Flex::row()
// 		.with_child(play_track_button())
// 		.with_default_spacer()
// 		.with_flex_child(
// 			Flex::column()
// 				.with_child(
// 					Label::new(|track: &Rc<Track>, _: &_| track.title.to_owned())
// 						.with_text_size(16.0)
// 						.fix_height(24.0)
// 						.expand_width(),
// 				)
// 				.with_child(EnvScope::new(
// 					|env, _| env.set(druid::theme::TEXT_COLOR, env.get(theme::FOREGROUND_DIM)),
// 					Label::new(|item: &Rc<Track>, _: &_| item.artist.to_owned())
// 						.with_text_size(13.0)
// 						.fix_height(10.0)
// 						.expand_width(),
// 				))
// 				.lens(Ctx::<TrackCtx, Rc<Track>>::data()),
// 			1.0,
// 		)
// 		.with_child(
// 			Painter::new(|ctx, _, env| draw_icon_button(ctx, env, ICON_EDIT))
// 				.fix_size(36.0, 36.0)
// 				.on_click(|ctx: &mut EventCtx, track: &mut Rc<Track>, _| {
// 					ctx.submit_command(command::UI_TRACK_EDIT_OPEN.with(track.id))
// 				})
// 				.lens(Ctx::<TrackCtx, Rc<Track>>::data()),
// 		)
// 		.with_child(
// 			Painter::new(|ctx, _, env| draw_icon_button(ctx, env, ICON_DELETE))
// 				.fix_size(36.0, 36.0)
// 				.on_click(|ctx: &mut EventCtx, track: &mut Rc<Track>, _| {
// 					let track_id = track.id;
// 					ctx.submit_command(overlay::SHOW_MODAL.with((
// 						Color::rgba(1.0, 1.0, 1.0, 0.1),
// 						Box::new(move |_| {
// 							Container::new(
// 								Flex::column()
// 									.with_child(Label::new("Delete this track?"))
// 									.with_default_spacer()
// 									.with_child(
// 										Flex::row()
// 											.with_child(
// 												FocusableButton::new("Cancel")
// 													.on_click(move |ctx, _, _| {
// 														ctx.submit_command(overlay::HIDE);
// 													})
// 													.controller(AutoFocus),
// 											)
// 											.with_default_spacer()
// 											.with_child(
// 												FocusableButton::new("Delete")
// 													.on_click(move |ctx, _, _| {
// 														ctx.submit_command(
// 															command::TRACK_DELETE.with(track_id),
// 														);
// 														ctx.submit_command(overlay::HIDE);
// 													})
// 													.env_scope(|env, _| {
// 														env.set(
// 															druid::theme::BUTTON_DARK,
// 															Color::RED,
// 														)
// 													}),
// 											),
// 									),
// 							)
// 							.padding(8.0)
// 							.background(theme::BACKGROUND)
// 							.boxed()
// 						}),
// 					)));
// 				})
// 				.lens(Ctx::<TrackCtx, Rc<Track>>::data()),
// 		)
// 		.expand_width()
// 		.fix_height(64.0);

// 	EnvScope::new(
// 		|env, state: &Ctx<_, _>| {
// 			env.set(
// 				TRACK_LIST_ITEM_BACKGROUND,
// 				if state.ctx.selected.as_deref() == Some(&state.data.id) {
// 					env.get(crate::theme::BACKGROUND_HIGHLIGHT0)
// 				} else {
// 					Color::TRANSPARENT
// 				},
// 			)
// 		},
// 		Container::new(row).background(TRACK_LIST_ITEM_BACKGROUND),
// 	)
// }

fn play_track_button() -> impl Widget<Ctx<TrackCtx, Rc<RefCell<Track>>>> {
	Painter::new(|ctx, data: &Ctx<TrackCtx, Rc<RefCell<Track>>>, env| {
		draw_icon_button(
			ctx,
			env,
			if data.ctx.playing.as_deref() == Some(&data.data.borrow().id) {
				ICON_PAUSE
			} else {
				ICON_PLAY
			},
		)
	})
	.fix_size(36.0, 36.0)
	.on_click(
		|ctx: &mut EventCtx, data: &mut Ctx<TrackCtx, Rc<RefCell<Track>>>, _| {
			if data.ctx.playing.as_deref() == Some(&data.data.borrow().id) {
				ctx.submit_command(PLAYER_PLAY_PAUSE);
			} else {
				ctx.submit_command(PLAYER_CLEAR);
				ctx.submit_command(PLAYER_ENQUEUE.with(data.data.borrow().clone()));
			}
		},
	)
}
