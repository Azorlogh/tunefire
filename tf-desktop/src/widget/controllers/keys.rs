use druid::{
	keyboard_types::Key, widget::Controller, Data, Env, Event, EventCtx, KeyEvent, Widget,
};

pub struct Enter<T> {
	action: Box<dyn Fn(&mut EventCtx, &mut T, &Env)>,
}
impl<T: Data> Enter<T> {
	pub fn new(action: impl Fn(&mut EventCtx, &mut T, &Env) + 'static) -> Self {
		Self {
			action: Box::new(action),
		}
	}
}
impl<T: Data, W: Widget<T>> Controller<T, W> for Enter<T> {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut T,
		env: &druid::Env,
	) {
		match event {
			Event::KeyDown(KeyEvent {
				key: Key::Enter, ..
			}) => {
				(self.action)(ctx, data, env);
			}
			_ => child.event(ctx, event, data, env),
		}
	}
}
