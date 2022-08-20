// use std::time::Duration;

// use druid::{widget::Controller, Env, Event, EventCtx, TimerToken, Widget};

// use crate::command;

// #[derive(Default)]
// pub struct PlayerTick {
// 	timer: Option<TimerToken>,
// }

// const DT: Duration = Duration::from_millis(500);

// impl<T, W: Widget<T>> Controller<T, W> for PlayerTick {
// 	fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
// 		match event {
// 			Event::Timer(t) if Some(*t) == self.timer => {
// 				ctx.submit_command(command::PLAYER_TICK);
// 				self.timer = Some(ctx.request_timer(DT));
// 			}
// 			Event::Command(c) if c.is(command::TRACK_PLAY) || c.is(command::QUERY_PLAY) => {
// 				self.timer = Some(ctx.request_timer(DT));
// 			}
// 			_ => {}
// 		}

// 		if let Event::WindowConnected = event {
// 			ctx.request_focus();
// 		}
// 		child.event(ctx, event, data, env);
// 	}
// }
