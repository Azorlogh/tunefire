use std::time::Duration;

use druid::{
	keyboard_types::Key,
	lens::{self, Field},
	widget::{Flex, Label, List, Maybe, SizedBox, TextBox},
	Color, Data, Env, Event, Point, TimerToken, Widget, WidgetExt, WidgetPod,
};

use super::{
	common::{
		dropdown::{self, Dropdown},
		dynamic_image::DynamicImage,
	},
	controllers::AutoFocus,
};
use crate::{
	command,
	controller::plugin,
	data::ctx::Ctx,
	plugins::SearchResult,
	state::{NewTrack, TrackSuggestions},
	theme,
};

const SUGGESTION_BACKGROUND: druid::Key<Color> = druid::Key::new("widget.suggestion.background");

pub type WData = Ctx<TrackSuggestions, String>;

pub struct SearchBar {
	inner: WidgetPod<WData, Box<dyn Widget<WData>>>,
	search_timer: TimerToken,
}

impl SearchBar {
	pub fn new() -> Self {
		Self {
			inner: WidgetPod::new(
				Dropdown::new_upward(
					TextBox::new().controller(AutoFocus).lens(Ctx::data()),
					|_, _| track_suggestions().lens(Ctx::ctx()),
					300.0,
				)
				.expand_width()
				.boxed(),
			),
			search_timer: TimerToken::INVALID,
		}
	}
}

impl Widget<WData> for SearchBar {
	fn event(
		&mut self,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut WData,
		env: &druid::Env,
	) {
		self.inner.event(ctx, event, data, env);
		match event {
			Event::KeyDown(event) if event.key == Key::ArrowUp => {
				data.ctx.selected = data.ctx.selected.saturating_sub(1);
				ctx.set_handled();
			}
			Event::KeyDown(event) if event.key == Key::ArrowDown => {
				data.ctx.selected = data
					.ctx
					.selected
					.saturating_add(1)
					.min(data.ctx.tracks.len());
				ctx.set_handled();
			}
			Event::KeyDown(event) if event.key == Key::Enter => {
				let suggestions = std::mem::take(&mut data.ctx.tracks);
				let new_track = if let Some(track) = data
					.ctx
					.selected
					.checked_sub(1)
					.and_then(|i| suggestions.into_iter().nth(i))
				{
					NewTrack {
						source: track.url.to_string(),
						artist: track.artist,
						title: track.title,
					}
				} else {
					NewTrack {
						source: data.data.to_owned(),
						artist: String::new(),
						title: String::new(),
					}
				};
				ctx.submit_command(command::UI_TRACK_ADD_OPEN.with(new_track));
				ctx.focus_next();
				ctx.submit_command(dropdown::DROPDOWN_HIDE.to(self.inner.id()));
			}
			Event::KeyDown(event) if event.key == Key::Escape => {
				ctx.submit_command(dropdown::DROPDOWN_HIDE.to(self.inner.id()));
			}
			Event::Timer(token) if token == &self.search_timer => {
				println!("SEARCH TRAKC REQURSEST");
				ctx.submit_command(plugin::SEARCH_TRACK_REQUEST.with(data.data.to_owned()));
			}
			_ => {}
		}
	}

	fn lifecycle(
		&mut self,
		ctx: &mut druid::LifeCycleCtx,
		event: &druid::LifeCycle,
		data: &WData,
		env: &druid::Env,
	) {
		self.inner.lifecycle(ctx, event, data, env);
	}

	fn update(
		&mut self,
		ctx: &mut druid::UpdateCtx,
		old_data: &WData,
		data: &WData,
		env: &druid::Env,
	) {
		self.inner.update(ctx, data, env);
		if !old_data.ctx.same(&data.ctx) {
			if data.ctx.tracks.is_empty() {
				ctx.submit_command(dropdown::DROPDOWN_HIDE.to(self.inner.id()));
			} else {
				ctx.submit_command(dropdown::DROPDOWN_SHOW.to(self.inner.id()))
			}
		}
		if !old_data.data.same(&data.data) {
			self.search_timer = ctx.request_timer(Duration::from_millis(300));
		}
	}

	fn layout(
		&mut self,
		ctx: &mut druid::LayoutCtx,
		bc: &druid::BoxConstraints,
		data: &WData,
		env: &druid::Env,
	) -> druid::Size {
		let size = self.inner.layout(ctx, bc, data, env);
		self.inner.set_origin(ctx, Point::ORIGIN);
		size
	}

	fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &WData, env: &druid::Env) {
		self.inner.paint(ctx, data, env);
	}
}

fn track_suggestions() -> impl Widget<TrackSuggestions> {
	List::new(|| {
		Flex::row()
			.with_child(
				Maybe::new(|| DynamicImage::new(), || SizedBox::empty())
					.fix_width(24.0)
					.fix_height(24.0)
					.lens(SearchResult::artwork)
					.lens(Field::new(|x: &(_, _)| &x.1, |x| &mut x.1))
					.lens(Ctx::data()),
			)
			.with_flex_child(
				Label::new(|data: &Ctx<Option<usize>, (usize, SearchResult)>, _: &_| {
					format!("{} - {}", data.data.1.artist, data.data.1.title)
				}),
				1.0,
			)
			.fix_width(300.0)
			.padding(8.0)
			.background(SUGGESTION_BACKGROUND)
			.env_scope(
				|env: &mut Env, state: &Ctx<Option<usize>, (usize, SearchResult)>| {
					env.set(
						SUGGESTION_BACKGROUND,
						if state.ctx == Some(state.data.0) {
							env.get(theme::BACKGROUND_HIGHLIGHT1)
						} else {
							Color::TRANSPARENT
						},
					)
				},
			)
	})
	.lens(Ctx::enumerate())
	.lens(Ctx::make(
		lens::Map::new(
			|s: &TrackSuggestions| s.selected.checked_sub(1),
			|_: &mut _, _| {},
		),
		TrackSuggestions::tracks,
	))
	.fix_height(300.0)
}
