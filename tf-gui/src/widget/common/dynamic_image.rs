use druid::{
	widget::Image, BoxConstraints, Data, Env, Event, EventCtx, ImageBuf, LayoutCtx, LifeCycle,
	LifeCycleCtx, PaintCtx, Size, UpdateCtx, Widget,
};

pub struct DynamicImage {
	inner: Image,
}

impl DynamicImage {
	pub fn new() -> DynamicImage {
		DynamicImage {
			inner: Image::new(ImageBuf::empty()),
		}
	}
}

impl Widget<ImageBuf> for DynamicImage {
	fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut ImageBuf, env: &Env) {
		self.inner.event(ctx, event, data, env)
	}

	fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &ImageBuf, env: &Env) {
		match event {
			LifeCycle::WidgetAdded => self.inner = Image::new(data.clone()),
			_ => {}
		}
		self.inner.lifecycle(ctx, event, data, env);
	}

	fn update(&mut self, ctx: &mut UpdateCtx, old_data: &ImageBuf, data: &ImageBuf, _env: &Env) {
		if !old_data.same(data) {
			self.inner = Image::new(data.clone());
			ctx.children_changed();
		}
	}

	fn layout(
		&mut self,
		ctx: &mut LayoutCtx,
		bc: &BoxConstraints,
		data: &ImageBuf,
		env: &Env,
	) -> Size {
		self.inner.layout(ctx, bc, data, env)
	}

	fn paint(&mut self, ctx: &mut PaintCtx, data: &ImageBuf, env: &Env) {
		self.inner.paint(ctx, data, env)
	}
}
