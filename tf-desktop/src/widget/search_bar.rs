use std::time::Duration;

use druid::{
	keyboard_types::Key,
	lens,
	widget::{Container, EnvScope, Flex, Label, List, TextBox},
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
	data::ctx::Ctx,
	plugins::SearchResult,
	state::{TagSuggestions, TrackSuggestions},
	theme, State,
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
				Dropdown::new(
					TextBox::new().controller(AutoFocus).lens(Ctx::data()),
					|_, _| track_suggestions().lens(Ctx::ctx()),
				)
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
					.min(data.ctx.tracks.len().saturating_sub(1));
				ctx.set_handled();
			}
			Event::KeyDown(event) if event.key == Key::Enter => {
				let suggestions = std::mem::take(&mut data.ctx.tracks);
				// if let Some(track) = suggestions.into_iter().nth(data.ctx.selected) {
				// 	data.data = track;
				// }
				ctx.focus_next();
				ctx.submit_command(dropdown::DROPDOWN_HIDE.to(self.inner.id()));
			}
			Event::KeyDown(event) if event.key == Key::Escape => {
				ctx.submit_command(dropdown::DROPDOWN_HIDE.to(self.inner.id()));
			}
			Event::Timer(token) if token == &self.search_timer => {
				ctx.submit_command(command::PLUGIN_SEARCH_TRACK.with(data.data.to_owned()));
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
			println!("{:?}", data.ctx.tracks);
			if data.ctx.tracks.is_empty() {
				ctx.submit_command(dropdown::DROPDOWN_HIDE.to(self.inner.id()));
			} else {
				ctx.submit_command(dropdown::DROPDOWN_SHOW.to(self.inner.id()))
			}
		}
		if !old_data.data.same(&data.data) {
			self.search_timer = ctx.request_timer(Duration::from_secs(1));
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
		self.inner.set_origin(ctx, data, env, Point::ORIGIN);
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
				DynamicImage::new()
					.fix_width(24.0)
					.fix_height(24.0)
					.lens(SearchResult::artwork),
			)
			.with_child(Label::new(
				|data: &Ctx<usize, (usize, SearchResult)>, _: &_| {
					format!("{} - {}", data.data.1.artist, data.data.1.title)
				},
			))
	})
	.lens(Ctx::enumerate())
	.lens(Ctx::make(
		lens::Map::new(|s: &TrackSuggestions| s.selected, |_, _| {}),
		TrackSuggestions::tracks,
	))
}
