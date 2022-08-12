use druid::{widget::Controller, Data, Env, Event, EventCtx, MouseButton, Widget};

pub struct ClickAfter<T> {
	action: Box<dyn Fn(&mut EventCtx, &mut T, &Env)>,
}
impl<T: Data> ClickAfter<T> {
	pub fn new(action: impl Fn(&mut EventCtx, &mut T, &Env) + 'static) -> Self {
		Self {
			action: Box::new(action),
		}
	}
}
impl<T: Data, W: Widget<T>> Controller<T, W> for ClickAfter<T> {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut T,
		env: &druid::Env,
	) {
		child.event(ctx, event, data, env);

		match event {
			Event::MouseDown(mouse_event) => {
				if mouse_event.button == MouseButton::Left
					&& !ctx.is_disabled()
					&& !ctx.is_handled()
				{
					ctx.set_active(true);
					ctx.request_paint();
				}
			}
			Event::MouseUp(mouse_event) => {
				if ctx.is_active() && mouse_event.button == MouseButton::Left && !ctx.is_handled() {
					ctx.set_active(false);
					if ctx.is_hot() && !ctx.is_disabled() {
						(self.action)(ctx, data, env);
					}
					ctx.request_paint();
				}
			}
			_ => {}
		}
	}
}

pub struct ClickBlocker;
impl<T: Data, W: Widget<T>> Controller<T, W> for ClickBlocker {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut T,
		env: &druid::Env,
	) {
		child.event(ctx, event, data, env);

		match event {
			Event::MouseDown(_) => {
				ctx.set_handled();
			}
			Event::MouseUp(_) => {
				ctx.set_handled();
			}
			_ => {}
		}
	}
}
