use std::rc::Rc;

use druid::{
	kurbo::{Line, Size},
	piet::{LineCap, LineJoin, RenderContext, StrokeStyle},
	widget::prelude::*,
	Point,
};
use tf_player::player;

use crate::{command, theme};

#[derive(Default)]
pub struct PlayerBar {
	position_preview: f64,
}

type Data = Rc<player::state::Playing>;

impl Widget<Data> for PlayerBar {
	fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Data, _env: &Env) {
		match event {
			Event::MouseDown(_) => {
				if !ctx.is_disabled() {
					ctx.set_active(true);
					ctx.request_paint();
				}
			}
			Event::MouseMove(evt) => {
				if ctx.is_active() {
					self.position_preview = evt.pos.x / ctx.size().width;
					ctx.request_paint();
				}
			}
			Event::MouseUp(evt) => {
				if ctx.is_active() && !ctx.is_disabled() {
					let position = data.track.duration.mul_f64(evt.pos.x / ctx.size().width);
					ctx.submit_command(command::PLAYER_SEEK.with(position));
					ctx.request_paint();
				}
				ctx.set_active(false);
			}
			_ => (),
		}
	}

	fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &Data, _env: &Env) {
		if let LifeCycle::HotChanged(_) | LifeCycle::DisabledChanged(_) = event {
			ctx.request_paint();
		}
	}

	fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &Data, _data: &Data, _env: &Env) {
		ctx.request_paint();
	}

	fn layout(
		&mut self,
		_ctx: &mut LayoutCtx,
		bc: &BoxConstraints,
		_data: &Data,
		_env: &Env,
	) -> Size {
		Size::new(bc.max().width, 24.0)
	}

	fn paint(&mut self, ctx: &mut PaintCtx, data: &Data, env: &Env) {
		let size = ctx.size();

		let left = Point::new(0.0, size.height / 2.0);

		let right = Point::new(size.width, size.height / 2.0);

		ctx.stroke_styled(
			Line::new(left, right),
			&env.get(theme::BACKGROUND_HIGHLIGHT0),
			6.0,
			&StrokeStyle {
				line_join: LineJoin::Round,
				line_cap: LineCap::Round,
				..Default::default()
			},
		);

		let progress = if ctx.is_active() {
			self.position_preview
		} else {
			data.offset.as_secs_f64() / data.track.duration.as_secs_f64()
		};
		let progress_right = Point::new(size.width * progress, size.height / 2.0);

		ctx.stroke_styled(
			Line::new(left, progress_right),
			&env.get(theme::ACCENT),
			6.0,
			&StrokeStyle {
				line_join: LineJoin::Round,
				line_cap: LineCap::Round,
				..Default::default()
			},
		);
	}
}
