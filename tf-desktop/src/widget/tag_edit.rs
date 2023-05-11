use druid::{lens, BoxConstraints, LensExt, Point, Size, Widget, WidgetExt, WidgetPod};

use super::{common::knob::Knob, tag_text_box::TagTextBox};
use crate::{data::ctx::Ctx, state::TagSuggestions, theme};

type Data = Ctx<TagSuggestions, (u128, (String, f32))>;

/// This widget is required because I want the Knob's side length to depend on the TextBox's height
/// AFAICT this isn't possible with simple flex layouts
pub struct TagEdit {
	text_box: WidgetPod<Data, Box<dyn Widget<Data>>>,
	knob: WidgetPod<Data, Box<dyn Widget<Data>>>,
}

impl TagEdit {
	pub fn new() -> Self {
		Self {
			text_box: WidgetPod::new(
				TagTextBox::new()
					.lens(Ctx::map((
						lens!((u128, (String, f32)), 0),
						lens!((u128, (String, f32)), 1.0),
					)))
					.boxed(),
			),
			knob: WidgetPod::new(
				Knob::new()
					.env_scope(|env, _| {
						env.set(druid::theme::FOREGROUND_DARK, env.get(theme::ACCENT))
					})
					.lens(Ctx::data().then(lens!((u128, (String, f32)), 1.1)))
					.boxed(),
			),
		}
	}
}

impl Widget<Data> for TagEdit {
	fn event(
		&mut self,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut Data,
		env: &druid::Env,
	) {
		self.text_box.event(ctx, event, data, env);
		self.knob.event(ctx, event, data, env);
	}

	fn lifecycle(
		&mut self,
		ctx: &mut druid::LifeCycleCtx,
		event: &druid::LifeCycle,
		data: &Data,
		env: &druid::Env,
	) {
		self.text_box.lifecycle(ctx, event, data, env);
		self.knob.lifecycle(ctx, event, data, env);
	}

	fn update(
		&mut self,
		ctx: &mut druid::UpdateCtx,
		_old_data: &Data,
		data: &Data,
		env: &druid::Env,
	) {
		self.text_box.update(ctx, data, env);
		self.knob.update(ctx, data, env);
	}

	fn layout(
		&mut self,
		ctx: &mut druid::LayoutCtx,
		bc: &druid::BoxConstraints,
		data: &Data,
		env: &druid::Env,
	) -> druid::Size {
		let text_box_size = self.text_box.layout(ctx, bc, data, env);
		let knob_bc = BoxConstraints::new(
			Size::new(0.0, 0.0),
			Size::new(text_box_size.height, text_box_size.height),
		);
		let knob_size = self.knob.layout(ctx, &knob_bc, data, env);

		let text_box_bc = BoxConstraints::tight(Size::new(
			text_box_size.width - knob_size.width,
			text_box_size.height,
		));
		let text_box_size = self.text_box.layout(ctx, &text_box_bc, data, env);

		self.text_box.set_origin(ctx, Point::new(0.0, 0.0));
		self.knob
			.set_origin(ctx, Point::new(text_box_size.width, 0.0));
		Size::new(
			text_box_size.width + knob_size.width,
			text_box_size.height.max(knob_size.height),
		)
	}

	fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &Data, env: &druid::Env) {
		self.text_box.paint(ctx, data, env);
		self.knob.paint(ctx, data, env);
	}
}
