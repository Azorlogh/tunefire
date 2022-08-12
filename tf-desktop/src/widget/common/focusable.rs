use druid::{
	debug_state::DebugState,
	widget::{prelude::*, Axis, WidgetWrapper},
	widget_wrapper_body,
};

pub struct Focusable<W> {
	widget: W,
}

impl<W> Focusable<W> {
	pub fn new(widget: W) -> Focusable<W> {
		Focusable { widget }
	}
}

impl<T, W: Widget<T>> Widget<T> for Focusable<W> {
	fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
		self.widget.event(ctx, event, data, env);
	}

	fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
		if let LifeCycle::BuildFocusChain = event {
			ctx.register_for_focus()
		}
		self.widget.lifecycle(ctx, event, data, env);
	}

	fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
		self.widget.update(ctx, old_data, data, env);
	}

	fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
		self.widget.layout(ctx, bc, data, env)
	}

	fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
		self.widget.paint(ctx, data, env)
	}

	fn id(&self) -> Option<WidgetId> {
		self.widget.id()
	}

	fn debug_state(&self, data: &T) -> DebugState {
		DebugState {
			display_name: self.short_type_name().to_string(),
			children: vec![self.widget.debug_state(data)],
			..Default::default()
		}
	}

	fn compute_max_intrinsic(
		&mut self,
		axis: Axis,
		ctx: &mut LayoutCtx,
		bc: &BoxConstraints,
		data: &T,
		env: &Env,
	) -> f64 {
		self.widget.compute_max_intrinsic(axis, ctx, bc, data, env)
	}
}

impl<W> WidgetWrapper for Focusable<W> {
	widget_wrapper_body!(W, widget);
}
