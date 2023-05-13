use std::time::Duration;

use druid::{
	widget::Controller, Data, Env, Event, EventCtx, LifeCycle, LifeCycleCtx, Selector, TimerToken,
	Widget,
};

const TAKE_FOCUS: Selector<()> = Selector::new("auto_focus.take_focus");

pub struct AutoFocus;

impl<W: Widget<T>, T> Controller<T, W> for AutoFocus {
	fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
		if let Event::Command(cmd) = event {
			if cmd.is(TAKE_FOCUS) {
				ctx.request_focus();
			}
		}
		child.event(ctx, event, data, env)
	}

	fn lifecycle(
		&mut self,
		child: &mut W,
		ctx: &mut LifeCycleCtx,
		event: &LifeCycle,
		data: &T,
		env: &Env,
	) {
		if let LifeCycle::WidgetAdded = event {
			ctx.submit_command(TAKE_FOCUS.to(ctx.widget_id()))
		}
		child.lifecycle(ctx, event, data, env)
	}
}

////////////

pub struct Focusable;

impl<W: Widget<T>, T> Controller<T, W> for Focusable {
	fn lifecycle(
		&mut self,
		child: &mut W,
		ctx: &mut LifeCycleCtx,
		event: &LifeCycle,
		data: &T,
		env: &Env,
	) {
		if let LifeCycle::BuildFocusChain = event {
			ctx.register_for_focus()
		}
		if let LifeCycle::FocusChanged(true) = event {}
		child.lifecycle(ctx, event, data, env)
	}
}

////////////

pub struct OnFocus<F> {
	when_gained: bool,
	handler: F,
	timer: TimerToken,
}

impl<F> OnFocus<F> {
	pub fn gained<T>(handler: F) -> Self
	where
		F: Fn(&mut EventCtx, &mut T, &Env),
	{
		Self {
			when_gained: true,
			handler,
			timer: TimerToken::INVALID,
		}
	}

	pub fn lost<T>(handler: F) -> Self
	where
		F: Fn(&mut EventCtx, &mut T, &Env),
	{
		Self {
			when_gained: false,
			handler,
			timer: TimerToken::INVALID,
		}
	}
}

impl<T, F, W> Controller<T, W> for OnFocus<F>
where
	T: Data,
	F: Fn(&mut EventCtx, &mut T, &Env),
	W: Widget<T>,
{
	fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
		match event {
			Event::Timer(timer) if *timer == self.timer => {
				(self.handler)(ctx, data, env);
				self.timer = TimerToken::INVALID;
				ctx.set_handled();
			}
			_ => {}
		}
		child.event(ctx, event, data, env);
	}
	fn lifecycle(
		&mut self,
		child: &mut W,
		ctx: &mut LifeCycleCtx,
		event: &LifeCycle,
		data: &T,
		env: &Env,
	) {
		if let LifeCycle::FocusChanged(focus) = event {
			if *focus == self.when_gained {
				self.timer = ctx.request_timer(Duration::from_millis(100));
			}
		}
		child.lifecycle(ctx, event, data, env);
	}
}
