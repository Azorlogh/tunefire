use std::f64::consts::TAU;

use druid::{
	kurbo::{BezPath, Size},
	piet::RenderContext,
	widget::prelude::*,
	Affine, Color, Point,
};
use palette::{FromColor, Gradient, IntoColor, Oklch, Srgb};

pub const STEPS: usize = 7;

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

fn quantize(x: f32) -> f32 {
	((x * STEPS as f32).floor() / STEPS as f32)
		.min(1.0)
		.max(0.0)
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
					self.value_preview =
						quantize(self.initial_data + (off.x - off.y) as f32 * SPEED);
					ctx.request_paint();
				}
			}
			Event::MouseUp(evt) => {
				if ctx.is_active() && !ctx.is_disabled() {
					let off = evt.pos - self.click;
					*data = quantize(self.initial_data + (off.x - off.y) as f32 * SPEED);
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

		let value = if ctx.is_active() {
			self.value_preview as f64
		} else {
			*data as f64
		};

		let color = |code: usize| {
			let color = palette::Srgb::new(
				(code >> 16) as u8 as f64 / 255.0,
				(code >> 8) as u8 as f64 / 255.0,
				code as u8 as f64 / 255.0,
			);
			// color.into_linear()
			Oklch::from_color(color)
		};

		let g = Gradient::new([color(0x98c379), color(0x61afef)]);
		// let g = Gradient::new([color(0xDC0E0E), color(0xFFEB33)]);

		for i in 0..STEPS {
			let v0 = i as f64 / STEPS as f64;
			let v1 = (i + 1) as f64 / STEPS as f64;
			if value < v1 {
				break;
			}
			let ang0 = v0 * TAU;
			let ang1 = v1 * TAU;
			let mut path = BezPath::new();
			path.line_to(Point::new(ang0.cos() * radius, ang0.sin() * radius));
			path.line_to(Point::new(ang1.cos() * radius, ang1.sin() * radius));
			path.line_to(Point::ZERO);
			path.close_path();
			path.apply_affine(Affine::translate(center.to_vec2()));
			let mut pos = i as f64 / (STEPS - 1) as f64;
			pos = (pos * TAU / 4.0).sin().powf(2.0);
			let color: Srgb<f64> = g.get(pos).into_color();
			ctx.fill(path, &Color::rgb(color.red, color.green, color.blue));
		}
		let mut path = BezPath::new();
		for i in 0..=STEPS {
			let ang0 = i as f64 / STEPS as f64 * TAU;
			let ang1 = (i + 1) as f64 / STEPS as f64 * TAU;
			path.line_to(Point::new(ang0.cos() * radius, ang0.sin() * radius));
			path.line_to(Point::new(ang1.cos() * radius, ang1.sin() * radius));
			path.move_to(Point::ZERO);
		}
		path.close_path();
		path.apply_affine(Affine::translate(center.to_vec2()));
		ctx.stroke(path, &env.get(crate::theme::BACKGROUND_HIGHLIGHT1), 1.0);
	}
}
