use std::{cell::RefCell, rc::Rc, time::Duration};

use druid::{
	lens,
	widget::{
		Axis, Container, CrossAxisAlignment, Either, EnvScope, Flex, Label, List,
		MainAxisAlignment, Painter, SizedBox,
	},
	Color, Data, EventCtx, Lens, RenderContext, Widget, WidgetExt,
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
		controllers::{AutoFocus, OnDebounce, OnHotChange},
		overlay,
	},
	State,
};

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

	let tag_columns = Flex::row()
		.with_child(
			List::new(|| {
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
									.with_child(Label::new(|s: &Ctx<_, String>, _: &_| {
										s.data.clone()
									}))
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
												.lens(Ctx::data())
												.controller(OnDebounce::trailing(
													Duration::from_secs(1),
													|ctx,
													 data: &mut Ctx<(Rc<Uuid>, String), f32>,
													 _| {
														// println!("that data changed! {data:?}");
														ctx.submit_command(
															command::TRACK_EDIT_TAG.with((
																*data.ctx.0,
																data.ctx.1.clone(),
																data.data,
															)),
														);
													},
												))
												.lens(Ctx::make(
													lens::Map::new(
														|s: &Ctx<String, Rc<RefCell<Track>>>| {
															(
																Rc::new(s.data.borrow().id),
																s.ctx.clone(),
															)
														},
														|_, _| {},
													),
													lens::Map::new(
														|s: &Ctx<String, Rc<RefCell<Track>>>| {
															*s.data
																.borrow()
																.tags
																.get(&s.ctx)
																.unwrap_or(&0.0)
														},
														|s: &mut Ctx<
															String,
															Rc<RefCell<Track>>,
														>,
														 inner: f32| {
															s.data
																.borrow_mut()
																.tags
																.insert(s.ctx.clone(), inner);
														},
													),
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
			.lens(Ctx::make(State::tracks, State::shown_tags)),
		)
		.with_child(
			Separator::new()
				.with_axis(Axis::Vertical)
				.with_width(1.0)
				.with_color(theme::BACKGROUND_HIGHLIGHT1)
				.fix_height(0.0),
		)
		.cross_axis_alignment(CrossAxisAlignment::Fill);

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
		.with_child(column_ui("", delete_button).fix_width(64.0))
		.with_default_spacer()
		.cross_axis_alignment(CrossAxisAlignment::Start);

	Stack::new()
		.with_child(
			Flex::column()
				.with_child(Label::new(""))
				.with_default_spacer()
				.with_child(
					List::new(|| {
						Painter::new(|ctx, data: &Ctx<TrackCtx, Rc<RefCell<Track>>>, env| {
							if ctx.is_hot()
								|| data.ctx.selected.as_deref() == Some(&data.data.borrow().id)
							{
								let size = ctx.size().to_rect();
								ctx.fill(size, &env.get(theme::BACKGROUND_HIGHLIGHT1))
							}
						})
						.controller(OnHotChange::new(|ctx, _, _, _| ctx.request_paint()))
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
				),
		)
		.with_child(table)
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

fn delete_button() -> impl Widget<Ctx<TrackCtx, Rc<RefCell<Track>>>> {
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
														command::TRACK_DELETE.with(track_id),
													);
													ctx.submit_command(overlay::HIDE);
												})
												.env_scope(|env, _| {
													env.set(druid::theme::BUTTON_DARK, Color::RED)
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
}
