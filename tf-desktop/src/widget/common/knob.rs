use std::f64::consts::TAU;

use druid::{
	kurbo::{self, BezPath, Shape, Size},
	piet::RenderContext,
	widget::prelude::*,
	Point, Vec2,
};

pub struct Knob {
	initial_data: f32,
	value_preview: f32,
	click: Point,
}

impl Knob {
	pub fn new() -> Self {
		Self {
			initial_data: 0.0,
			value_preview: 0.0,
			click: Point::ORIGIN,
		}
	}
}

const SPEED: f32 = 10e-3;

impl Widget<f32> for Knob {
	fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut f32, _env: &Env) {
		match event {
			Event::MouseDown(evt) => {
				if !ctx.is_disabled() {
					ctx.set_active(true);
					self.click = evt.pos;
					self.value_preview = *data;
					self.initial_data = *data;
					ctx.request_paint();
				}
			}
			Event::MouseMove(evt) => {
				if ctx.is_active() {
					let off = evt.pos - self.click;
					self.value_preview = (self.initial_data + (off.x - off.y) as f32 * SPEED)
						.min(1.0)
						.max(0.0);
					ctx.request_paint();
				}
			}
			Event::MouseUp(evt) => {
				if ctx.is_active() && !ctx.is_disabled() {
					let off = evt.pos - self.click;
					*data = (self.initial_data + (off.x - off.y) as f32 * SPEED)
						.min(1.0)
						.max(0.0);
					ctx.request_paint();
				}
				ctx.set_active(false);
			}
			_ => (),
		}
	}

	fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &f32, _env: &Env) {
		if let LifeCycle::HotChanged(_) | LifeCycle::DisabledChanged(_) = event {
			ctx.request_paint();
		}
	}

	fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &f32, _data: &f32, _env: &Env) {
		ctx.request_paint();
	}

	fn layout(
		&mut self,
		_ctx: &mut LayoutCtx,
		bc: &BoxConstraints,
		_data: &f32,
		_env: &Env,
	) -> Size {
		bc.max()
	}

	fn paint(&mut self, ctx: &mut PaintCtx, data: &f32, env: &Env) {
		let size = ctx.size();

		let center = (size / 2.0).to_vec2().to_point();
		let radius = size.width / 2.0 * 0.7;

		let arc_path: BezPath = kurbo::Arc {
			center,
			radii: Vec2::new(radius, radius),
			start_angle: 0.0,
			sweep_angle: if ctx.is_active() {
				self.value_preview as f64 * TAU
			} else {
				*data as f64 * TAU
			},
			x_rotation: -TAU / 4.0,
		}
		.into_path(0.1);

		let mut wheel = arc_path.clone();
		wheel.line_to(center);
		wheel.line_to(center - Vec2::new(0.0, radius));

		ctx.fill(wheel, &env.get(crate::theme::ACCENT));
		ctx.stroke(arc_path, &env.get(crate::theme::ACCENT_DIM), 2.0);
	}
}
