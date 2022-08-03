use druid::{widget::prelude::*, WidgetPod};

pub struct Square<T> {
	size: f64,
	child: WidgetPod<T, Box<dyn Widget<T>>>,
}

impl<T: Data> Square<T> {
	pub fn new(child: impl Widget<T> + 'static) -> Self {
		Self {
			size: 0.0,
			child: WidgetPod::new(child).boxed(),
		}
	}
}

impl<T: Data> Widget<T> for Square<T> {
	fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
		self.child.event(ctx, event, data, env)
	}

	fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
		self.child.lifecycle(ctx, event, data, env)
	}

	fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
		self.child.update(ctx, data, env)
	}

	fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
		if !bc.is_width_bounded() && !bc.is_height_bounded() {
		} else {
			let max = bc.max();
			if max.width != 0.0 {
				self.size = max.width;
			}
			if max.height != 0.0 {
				self.size = max.height;
			}
		}
		let size = Size::new(self.size, self.size);
		self.child
			.layout(ctx, &BoxConstraints::new(size, size), data, env);
		size
	}

	fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
		self.child.paint(ctx, data, env)
	}
}
