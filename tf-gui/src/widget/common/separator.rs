// Copyright 2020 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A separator widget.

use druid::{
	kurbo::Line,
	piet::StrokeStyle,
	theme,
	widget::{prelude::*, Axis},
	Color, KeyOrValue,
};

/// A separator widget.
pub struct Separator {
	width: KeyOrValue<f64>,
	color: KeyOrValue<Color>,
	axis: Axis,
	stroke_style: StrokeStyle,
}

impl Default for Separator {
	fn default() -> Self {
		Separator {
			width: theme::BUTTON_BORDER_WIDTH.into(),
			color: theme::BORDER_LIGHT.into(),
			axis: Axis::Horizontal,
			stroke_style: StrokeStyle::new(),
		}
	}
}

impl Separator {
	pub fn new() -> Self {
		Self::default()
	}

	/// Set the separator width (thickness).
	pub fn with_width(mut self, width: impl Into<KeyOrValue<f64>>) -> Self {
		self.width = width.into();
		self
	}

	/// Set the separator width (thickness).
	pub fn set_width(&mut self, width: impl Into<KeyOrValue<f64>>) {
		self.width = width.into();
	}

	pub fn with_color(mut self, color: impl Into<KeyOrValue<Color>>) -> Self {
		self.color = color.into();
		self
	}

	pub fn set_color(&mut self, color: impl Into<KeyOrValue<Color>>) {
		self.color = color.into();
	}

	pub fn with_stroke_style(mut self, stroke_style: StrokeStyle) -> Self {
		self.stroke_style = stroke_style;
		self
	}

	pub fn set_stroke_style(&mut self, stroke_style: StrokeStyle) {
		self.stroke_style = stroke_style;
	}

	pub fn with_axis(mut self, axis: Axis) -> Self {
		self.axis = axis;
		self
	}

	pub fn set_axis(&mut self, axis: Axis) {
		self.axis = axis;
	}
}

impl<T> Widget<T> for Separator {
	fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut T, _env: &Env) {}

	fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {}

	fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

	fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
		let width = self.width.resolve(env);
		let size = match self.axis {
			Axis::Vertical => (width, f64::INFINITY),
			Axis::Horizontal => (f64::INFINITY, width),
		};
		bc.constrain(size)
	}

	fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
		let major_size = self.axis.major(ctx.size());
		let minor_size = self.axis.minor(ctx.size());
		let line = Line::new(
			self.axis.pack(0., minor_size / 2.0),
			self.axis.pack(major_size, minor_size / 2.0),
		);
		let color = self.color.resolve(env);
		let width = self.width.resolve(env);
		ctx.stroke_styled(line, &color, width, &self.stroke_style);
	}
}
