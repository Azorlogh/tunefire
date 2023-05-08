use druid::{
	BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
	Point, RenderContext, Selector, Size, UpdateCtx, Widget, WidgetPod,
};

use crate::state::State;

type ChildBuilder = Box<dyn Fn(&Env) -> Box<dyn Widget<State>>>;

pub const SHOW_AT: Selector<(Point, BoxConstraints, ChildBuilder)> =
	Selector::new("dropdown.show-at");
pub const SHOW_MIDDLE: Selector<(BoxConstraints, ChildBuilder)> =
	Selector::new("dropdown.show-middle");
pub const SHOW_MODAL: Selector<(Color, ChildBuilder)> = Selector::new("dropdown.show-modal");
pub const HIDE: Selector = Selector::new("dropdown.hide");

pub struct Child {
	origin: Option<Point>,
	bc: BoxConstraints,
	widget: WidgetPod<State, Box<dyn Widget<State>>>,
}

pub enum Overlay {
	Inactive,
	Active { child: Child, background: Color },
}

impl Overlay {
	pub fn new() -> Overlay {
		Overlay::Inactive
	}
}

impl Widget<State> for Overlay {
	fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, env: &Env) {
		let mut remove_child = false;
		if let Overlay::Active { child, .. } = self {
			// println!("{event:?}");
			child.widget.event(ctx, event, data, env);
			match event {
				Event::MouseDown(mouse) => {
					if !child.widget.layout_rect().contains(mouse.pos) {
						ctx.set_active(true);
					}
					ctx.set_handled();
				}
				Event::MouseUp(mouse) => {
					if ctx.is_active() && !child.widget.layout_rect().contains(mouse.pos) {
						remove_child = true;
						ctx.set_active(false);
					}
					ctx.set_handled();
				}
				// TODO: why am I not receiving this event?
				Event::KeyDown(evt) if evt.key == druid::keyboard_types::Key::Escape => {
					remove_child = true;
					ctx.set_active(false);
					ctx.set_handled();
				}
				Event::MouseMove(_) => {
					ctx.set_handled();
				}
				_ => {}
			}
		}
		match event {
			Event::Command(cmd) if cmd.is(SHOW_AT) => {
				let (pos, bc, child_builder) = cmd.get_unchecked(SHOW_AT);
				*self = Overlay::Active {
					child: Child {
						origin: Some(*pos),
						bc: *bc,
						widget: WidgetPod::new(child_builder(env)),
					},
					background: Color::TRANSPARENT,
				};
				ctx.children_changed();
			}
			Event::Command(cmd) if cmd.is(SHOW_MIDDLE) => {
				let (bc, child_builder) = cmd.get_unchecked(SHOW_MIDDLE);
				*self = Overlay::Active {
					child: Child {
						origin: None,
						bc: *bc,
						widget: WidgetPod::new(child_builder(env)),
					},
					background: Color::TRANSPARENT,
				};
				ctx.children_changed();
			}
			Event::Command(cmd) if cmd.is(SHOW_MODAL) => {
				let (background, child_builder) = cmd.get_unchecked(SHOW_MODAL);
				*self = Overlay::Active {
					child: Child {
						origin: None,
						bc: BoxConstraints::tight(ctx.size()).loosen(),
						widget: WidgetPod::new(child_builder(env)),
					},
					background: background.clone(),
				};
				ctx.children_changed();
			}
			Event::Command(cmd) if cmd.is(HIDE) => {
				remove_child = true;
			}
			_ => {}
		}
		if remove_child {
			*self = Overlay::Inactive;
			ctx.request_layout();
			ctx.request_paint();
		}
	}

	fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &State, env: &Env) {
		if let Overlay::Active { child, .. } = self {
			child.widget.lifecycle(ctx, event, data, env);
		}
	}

	fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &State, data: &State, env: &Env) {
		if let Overlay::Active { child, .. } = self {
			child.widget.update(ctx, data, env);
		}
	}

	fn layout(
		&mut self,
		ctx: &mut LayoutCtx,
		bc: &BoxConstraints,
		data: &State,
		env: &Env,
	) -> Size {
		if let Overlay::Active { child, .. } = self {
			let size = child.widget.layout(ctx, &child.bc, data, env);
			let origin = child
				.origin
				.unwrap_or_else(|| (bc.max().to_vec2() / 2.0 - size.to_vec2() / 2.0).to_point());
			child.widget.set_origin(ctx, origin);
		}
		bc.max()
	}

	fn paint(&mut self, ctx: &mut PaintCtx, data: &State, env: &Env) {
		let size = ctx.size();
		if let Overlay::Active { child, background } = self {
			ctx.fill(size.to_rect(), background);
			child.widget.paint(ctx, data, env);
		}
	}
}
