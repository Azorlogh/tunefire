use druid::{widget::Controller, Data, Env, LifeCycleCtx, Widget};

pub struct OnHotChange<F> {
	handler: F,
}

impl<F> OnHotChange<F> {
	pub fn new<T>(handler: F) -> Self
	where
		F: Fn(&mut LifeCycleCtx, &T, bool, &Env),
	{
		Self { handler }
	}
}

impl<T, F, W> Controller<T, W> for OnHotChange<F>
where
	T: Data,
	F: Fn(&mut LifeCycleCtx, &T, bool, &Env),
	W: Widget<T>,
{
	fn lifecycle(
		&mut self,
		child: &mut W,
		ctx: &mut LifeCycleCtx,
		event: &druid::LifeCycle,
		data: &T,
		env: &Env,
	) {
		child.lifecycle(ctx, event, data, env);
		match event {
			druid::LifeCycle::HotChanged(h) => (self.handler)(ctx, data, *h, env),
			_ => {}
		}
	}

	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut T,
		env: &Env,
	) {
		child.event(ctx, event, data, env)
	}
}
