use druid::{widget::Controller, Env, Event, EventCtx, LifeCycle, LifeCycleCtx, Selector, Widget};

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
