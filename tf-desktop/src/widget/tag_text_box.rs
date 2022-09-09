use druid::{widget::TextBox, BoxConstraints, Point, Size, Widget, WidgetExt, WidgetPod};

type Data = String;

pub struct TagTextBox {
	text_box: TextBox<String>,
}

impl TagTextBox {
	pub fn new() -> Self {
		Self {
			text_box: TextBox::new(),
		}
	}
}

impl Widget<Data> for TagTextBox {
	fn event(
		&mut self,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut Data,
		env: &druid::Env,
	) {
		self.text_box.event(ctx, event, data, env);
		if let druid::Event::KeyUp(k) = event {
			// if data == "a" {
			// }
		}
	}

	fn lifecycle(
		&mut self,
		ctx: &mut druid::LifeCycleCtx,
		event: &druid::LifeCycle,
		data: &Data,
		env: &druid::Env,
	) {
		self.text_box.lifecycle(ctx, event, data, env);
	}

	fn update(
		&mut self,
		ctx: &mut druid::UpdateCtx,
		old_data: &Data,
		data: &Data,
		env: &druid::Env,
	) {
		self.text_box.update(ctx, old_data, data, env);
	}

	fn layout(
		&mut self,
		ctx: &mut druid::LayoutCtx,
		bc: &druid::BoxConstraints,
		data: &Data,
		env: &druid::Env,
	) -> druid::Size {
		self.text_box.layout(ctx, bc, data, env)
	}

	fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &Data, env: &druid::Env) {
		self.text_box.paint(ctx, data, env);
	}
}
