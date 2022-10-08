// use druid::{
// 	debug_state::DebugState, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
// 	LifeCycleCtx, PaintCtx, Point, Size, UpdateCtx, Widget, WidgetPod,
// };
// pub struct HiddenIf<T, W> {
// 	child: Option<WidgetPod<T, W>>,
// 	hidden_if: Box<dyn Fn(&T, &Env) -> bool>,
// }

// impl<T: Data, W: Widget<T>> HiddenIf<T, W> {
// 	pub fn new(widget: W, hidden_if: impl Fn(&T, &Env) -> bool + 'static) -> Self {
// 		HiddenIf {
// 			child: WidgetPod::new(widget),
// 			hidden_if: Box::new(hidden_if),
// 		}
// 	}
// }

// impl<T: Data, W: Widget<T>> Widget<T> for HiddenIf<T, W> {
// 	fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
// 		self.child.event(ctx, event, data, env);
// 	}

// 	fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
// 		if let LifeCycle::WidgetAdded = event {
// 			ctx.set_disabled((self.hidden_if)(data, env));
// 		}
// 		self.child.lifecycle(ctx, event, data, env);
// 	}

// 	fn update(&mut self, ctx: &mut UpdateCtx, _: &T, data: &T, env: &Env) {
// 		ctx.set_disabled((self.hidden_if)(data, env));
// 		self.child.update(ctx, data, env);
// 	}

// 	fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
// 		let size = self.child.layout(ctx, bc, data, env);
// 		self.child.set_origin(ctx, data, env, Point::ZERO);
// 		ctx.set_baseline_offset(self.child.baseline_offset());
// 		size
// 	}

// 	fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
// 		self.child.paint(ctx, data, env);
// 	}

// 	fn debug_state(&self, data: &T) -> DebugState {
// 		DebugState {
// 			display_name: self.short_type_name().to_string(),
// 			children: vec![self.child.widget().debug_state(data)],
// 			..Default::default()
// 		}
// 	}
// }
