use druid::{keyboard_types::Key, widget::Controller, Data, Env, Event, EventCtx, Widget};

pub struct OnKey<T> {
	key: Key,
	action: Box<dyn Fn(&mut EventCtx, &mut T, &Env)>,
}
impl<T: Data> OnKey<T> {
	pub fn new(key: Key, action: impl Fn(&mut EventCtx, &mut T, &Env) + 'static) -> Self {
		Self {
			key,
			action: Box::new(action),
		}
	}
}
impl<T: Data, W: Widget<T>> Controller<T, W> for OnKey<T> {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut T,
		env: &druid::Env,
	) {
		match event {
			Event::KeyDown(evt) if evt.key == self.key => {
				(self.action)(ctx, data, env);
			}
			_ => child.event(ctx, event, data, env),
		}
	}
}
