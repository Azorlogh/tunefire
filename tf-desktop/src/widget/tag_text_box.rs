use druid::{
	keyboard_types::Key,
	lens,
	widget::{Container, EnvScope, Label, List, TextBox},
	Color, Data, Env, Event, Point, Widget, WidgetExt, WidgetPod,
};

use super::{
	common::dropdown::{self, Dropdown},
	controllers::AutoFocus,
};
use crate::{command, data::ctx::Ctx, state::TagSuggestions, theme};

const SUGGESTION_BACKGROUND: druid::Key<Color> = druid::Key::new("widget.suggestion.background");

pub type WData = Ctx<TagSuggestions, String>;

pub struct TagTextBox {
	inner: WidgetPod<WData, Box<dyn Widget<WData>>>,
}

impl TagTextBox {
	pub fn new() -> Self {
		Self {
			inner: WidgetPod::new(
				Dropdown::new(
					TextBox::new().controller(AutoFocus).lens(Ctx::data()),
					|_, _| {
						List::new(|| {
							EnvScope::new(
								|env: &mut Env, state: &Ctx<usize, (usize, String)>| {
									env.set(
										SUGGESTION_BACKGROUND,
										if state.ctx == state.data.0 {
											env.get(theme::BACKGROUND_HIGHLIGHT1)
										} else {
											Color::TRANSPARENT
										},
									)
								},
								Container::new(Label::new(|data: &(usize, String), _env: &Env| {
									data.1.clone()
								}))
								.padding(10.0)
								.background(SUGGESTION_BACKGROUND)
								.lens(Ctx::data()),
							)
						})
						.lens(Ctx::enumerate())
						.lens(Ctx::make(
							lens::Map::new(|s: &TagSuggestions| s.selected, |_, _| {}),
							TagSuggestions::tags,
						))
						.lens(Ctx::ctx())
					},
				)
				.boxed(),
			),
		}
	}
}

impl Widget<WData> for TagTextBox {
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
					.min(data.ctx.tags.len().saturating_sub(1));
				ctx.set_handled();
			}
			Event::KeyDown(event) if event.key == Key::Enter => {
				let suggestions = std::mem::take(&mut data.ctx.tags);
				if let Some(tag) = suggestions.into_iter().nth(data.ctx.selected) {
					data.data = tag;
				}
				ctx.focus_next();
				ctx.submit_command(dropdown::DROPDOWN_HIDE.to(self.inner.id()));
			}
			Event::KeyDown(event) if event.key == Key::Escape => {
				ctx.submit_command(dropdown::DROPDOWN_HIDE.to(self.inner.id()));
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
		if data.ctx.tags.is_empty() {
			ctx.submit_command(dropdown::DROPDOWN_HIDE.to(self.inner.id()));
		}
		if !old_data.data.same(&data.data) {
			if data.ctx.tags.len() != 0 {
				ctx.submit_command(dropdown::DROPDOWN_SHOW.to(self.inner.id()))
			}
			ctx.submit_command(command::TAG_SEARCH.with(data.data.clone()));
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
