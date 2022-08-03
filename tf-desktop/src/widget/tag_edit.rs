use druid::{
	text::ParseFormatter, widget::TextBox, BoxConstraints, Point, Size, Widget, WidgetExt,
	WidgetPod,
};

use super::common::knob::Knob;
use crate::{command, theme};

type Data = (String, f32);

pub struct TagEdit {
	text_box: WidgetPod<String, Box<dyn Widget<String>>>,
	knob: WidgetPod<f32, Box<dyn Widget<f32>>>,
}

impl TagEdit {
	pub fn new() -> Self {
		Self {
			text_box: WidgetPod::new(
				TextBox::new()
					.with_formatter(ParseFormatter::new())
					.update_data_while_editing(false)
					.boxed(),
			),
			knob: WidgetPod::new(
				Knob::new()
					.env_scope(|env, _| {
						env.set(druid::theme::FOREGROUND_DARK, env.get(theme::ACCENT))
					})
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
		let initial = data.clone();

		self.text_box.event(ctx, event, &mut data.0, env);
		self.knob.event(ctx, event, &mut data.1, env);
		if data.0 != initial.0 {
			ctx.submit_command(command::TAG_RENAME.with((initial.0, data.0.clone())));
		}

		if data.1 != initial.1 {
			ctx.submit_command(command::TAG_TWEAK.with((data.0.clone(), data.1)));
		}
	}

	fn lifecycle(
		&mut self,
		ctx: &mut druid::LifeCycleCtx,
		event: &druid::LifeCycle,
		data: &Data,
		env: &druid::Env,
	) {
		self.text_box.lifecycle(ctx, event, &data.0, env);
		self.knob.lifecycle(ctx, event, &data.1, env);
	}

	fn update(
		&mut self,
		ctx: &mut druid::UpdateCtx,
		_old_data: &Data,
		data: &Data,
		env: &druid::Env,
	) {
		self.text_box.update(ctx, &data.0, env);
		self.knob.update(ctx, &data.1, env);
	}

	fn layout(
		&mut self,
		ctx: &mut druid::LayoutCtx,
		bc: &druid::BoxConstraints,
		data: &Data,
		env: &druid::Env,
	) -> druid::Size {
		let text_box_size = self.text_box.layout(ctx, bc, &data.0, env);
		let knob_bc = BoxConstraints::new(
			Size::new(0.0, 0.0),
			Size::new(text_box_size.height, text_box_size.height),
		);
		let knob_size = self.knob.layout(ctx, &knob_bc, &data.1, env);

		let text_box_bc = BoxConstraints::tight(Size::new(
			text_box_size.width - knob_size.width,
			text_box_size.height,
		));
		let text_box_size = self.text_box.layout(ctx, &text_box_bc, &data.0, env);

		self.text_box
			.set_origin(ctx, &data.0, env, Point::new(0.0, 0.0));
		self.knob
			.set_origin(ctx, &data.1, env, Point::new(text_box_size.width, 0.0));
		Size::new(
			text_box_size.width + knob_size.width,
			text_box_size.height.max(knob_size.height),
		)
	}

	fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &Data, env: &druid::Env) {
		self.text_box.paint(ctx, &data.0, env);
		self.knob.paint(ctx, &data.1, env);
	}
}
