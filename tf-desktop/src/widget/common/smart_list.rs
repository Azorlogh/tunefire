// Copyright 2019 The Druid Authors.
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

//! Simple list view widget.

use std::{
	collections::{BTreeMap, BTreeSet},
	f64,
};

use druid::{
	debug_state::DebugState,
	im,
	kurbo::{Point, Rect, Size},
	widget::{Axis, ListIter},
	BoxConstraints, Data, Env, Event, EventCtx, KeyOrValue, LayoutCtx, LifeCycle, LifeCycleCtx,
	PaintCtx, Selector, UpdateCtx, Widget, WidgetPod,
};
use tracing::{instrument, trace};

#[cfg(feature = "im")]
use crate::im::{OrdMap, Vector};

pub type ItemId = u128;
pub type IdentifiedVector<T> = im::Vector<(u128, T)>; // todo: flesh this out
pub const ITEM_DELETE: Selector<u128> = Selector::new("identified-vector.item.delete");

/// A list widget for a variable-size collection of items.
pub struct SmartList<T> {
	closure: Box<dyn Fn() -> Box<dyn Widget<T>>>,
	children: BTreeMap<ItemId, WidgetPod<T, Box<dyn Widget<T>>>>,
	get_id: Box<dyn Fn(&T) -> ItemId>,
	axis: Axis,
	spacing: KeyOrValue<f64>,
	old_bc: BoxConstraints,
}

impl<T: Data> SmartList<T> {
	/// Create a new list widget. Closure will be called every time when a new child
	/// needs to be constructed.
	pub fn new<W: Widget<T> + 'static>(
		closure: impl Fn() -> W + 'static,
		get_id: impl Fn(&T) -> ItemId + 'static,
	) -> Self {
		SmartList {
			closure: Box::new(move || Box::new(closure())),
			children: BTreeMap::default(),
			get_id: Box::new(move |data| get_id(data)),
			axis: Axis::Vertical,
			spacing: KeyOrValue::Concrete(0.),
			old_bc: BoxConstraints::tight(Size::ZERO),
		}
	}

	/// Sets the widget to display the list horizontally, not vertically.
	pub fn horizontal(mut self) -> Self {
		self.axis = Axis::Horizontal;
		self
	}

	/// Set the spacing between elements.
	pub fn with_spacing(mut self, spacing: impl Into<KeyOrValue<f64>>) -> Self {
		self.spacing = spacing.into();
		self
	}

	/// Set the spacing between elements.
	pub fn set_spacing(&mut self, spacing: impl Into<KeyOrValue<f64>>) -> &mut Self {
		self.spacing = spacing.into();
		self
	}

	// /// When the widget is created or the data changes, create or remove children as needed
	// ///
	// /// Returns `true` if children were added or removed.
	// fn update_child_count(&mut self, data: &impl ListIter<(usize, T)>, _env: &Env) -> bool {
	// 	let len = self.children.len();
	// 	match len.cmp(&data.data_len()) {
	// 		Ordering::Greater => self.children.truncate(data.data_len()),
	// 		Ordering::Less => data.for_each(|_, i| {
	// 			if i >= len {
	// 				let child = WidgetPod::new((self.closure)());
	// 				self.children.push(child);
	// 			}
	// 		}),
	// 		Ordering::Equal => (),
	// 	}
	// 	len != data.data_len()
	// }

	fn update_child_count(&mut self, data: &impl ListIter<T>, _env: &Env) -> bool {
		let mut remaining: BTreeSet<u128> = self.children.keys().cloned().collect();
		let mut changed = false;
		data.for_each(|child_data, _| {
			let id = (self.get_id)(child_data);
			if !remaining.remove(&id) {
				self.children.insert(id, WidgetPod::new((self.closure)()));
				changed = true;
			}
		});
		for id in remaining {
			self.children.remove(&id);
			changed = true;
		}
		changed
	}
}

impl<C: Data, T: ListIter<C>> Widget<T> for SmartList<C> {
	#[instrument(name = "List", level = "trace", skip(self, ctx, event, data, env))]
	fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
		data.for_each_mut(|child_data, _| {
			let id = (self.get_id)(child_data);
			if let Some(child) = self.children.get_mut(&id) {
				child.event(ctx, event, child_data, env);
			}
		});
	}

	#[instrument(name = "List", level = "trace", skip(self, ctx, event, data, env))]
	fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
		if let LifeCycle::WidgetAdded = event {
			if self.update_child_count(data, env) {
				ctx.children_changed();
			}
		}

		data.for_each(|child_data, _| {
			let id = (self.get_id)(child_data);
			if let Some(child) = self.children.get_mut(&id) {
				child.lifecycle(ctx, event, child_data, env);
			}
		});
	}

	#[instrument(name = "List", level = "trace", skip(self, ctx, _old_data, data, env))]
	fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
		// we send update to children first, before adding or removing children;
		// this way we avoid sending update to newly added children, at the cost
		// of potentially updating children that are going to be removed.
		data.for_each(|child_data, _| {
			let id = (self.get_id)(child_data);
			if let Some(child) = self.children.get_mut(&id) {
				child.update(ctx, child_data, env);
			}
		});

		if self.update_child_count(data, env) {
			ctx.children_changed();
		}

		if ctx.env_key_changed(&self.spacing) {
			ctx.request_layout();
		}
	}

	#[instrument(name = "List", level = "trace", skip(self, ctx, bc, data, env))]
	fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
		let axis = self.axis;
		let spacing = self.spacing.resolve(env);
		let mut minor = axis.minor(bc.min());
		let mut major_pos = 0.0;
		let mut paint_rect = Rect::ZERO;

		let bc_changed = self.old_bc != *bc;
		self.old_bc = *bc;

		let child_bc = constraints(axis, bc, 0., f64::INFINITY);
		data.for_each(|child_data, _| {
			let id = (self.get_id)(child_data);
			let child = match self.children.get_mut(&id) {
				Some(child) => child,
				None => {
					return;
				}
			};

			let child_size = if bc_changed || child.layout_requested() {
				child.layout(ctx, &child_bc, child_data, env)
			} else {
				child.layout_rect().size()
			};

			let child_pos: Point = axis.pack(major_pos, 0.).into();
			child.set_origin(ctx, child_pos);
			paint_rect = paint_rect.union(child.paint_rect());
			minor = minor.max(axis.minor(child_size));
			major_pos += axis.major(child_size) + spacing;
		});

		// correct overshoot at end.
		major_pos -= spacing;

		let my_size = bc.constrain(Size::from(axis.pack(major_pos, minor)));
		let insets = paint_rect - my_size.to_rect();
		ctx.set_paint_insets(insets);
		trace!("Computed layout: size={}, insets={:?}", my_size, insets);
		my_size
	}

	#[instrument(name = "List", level = "trace", skip(self, ctx, data, env))]
	fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
		data.for_each(|child_data, _| {
			let id = (self.get_id)(child_data);
			if let Some(child) = self.children.get_mut(&id) {
				child.paint(ctx, child_data, env);
			}
		});
	}

	fn debug_state(&self, data: &T) -> DebugState {
		let mut children_state = Vec::with_capacity(data.data_len());
		data.for_each(|child_data, _| {
			let id = (self.get_id)(child_data);
			if let Some(child) = self.children.get(&id) {
				children_state.push(child.widget().debug_state(child_data));
			}
		});

		DebugState {
			display_name: "List".to_string(),
			children: children_state,
			..Default::default()
		}
	}
}

pub fn constraints(axis: Axis, bc: &BoxConstraints, min_major: f64, major: f64) -> BoxConstraints {
	match axis {
		Axis::Horizontal => BoxConstraints::new(
			Size::new(min_major, bc.min().height),
			Size::new(major, bc.max().height),
		),
		Axis::Vertical => BoxConstraints::new(
			Size::new(bc.min().width, min_major),
			Size::new(bc.max().width, major),
		),
	}
}
