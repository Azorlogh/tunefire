use druid::{
	commands::CLOSE_WINDOW,
	widget::{prelude::*, Controller, WidgetExt},
	Point, Target, WindowConfig, WindowId, WindowLevel, WindowSizePolicy,
};

type DropFn<T> = Box<dyn Fn(&T, &Env) -> Box<dyn Widget<T>>>;

enum Positioning {
	Down,
	Up(f64),
}

pub struct Dropdown<T> {
	drop: DropFn<T>,
	window: Option<WindowId>,
	positioning: Positioning,
}

pub const DROPDOWN_SHOW: druid::Selector = druid::Selector::new("dropdown.show");
pub const DROPDOWN_HIDE: druid::Selector = druid::Selector::new("dropdown.hide");
pub const DROPDOWN_CLOSED: druid::Selector = druid::Selector::new("dropdown.closed");

impl<T: Data> Dropdown<T> {
	pub fn new<W: 'static + Widget<T>, DW: Widget<T> + 'static>(
		header: W,
		make_drop: impl Fn(&T, &Env) -> DW + 'static,
	) -> impl Widget<T> {
		// padding for putting header in separate WidgetPod
		// because notifications from same WidgetPod are not sent
		header.padding(0.).controller(Dropdown {
			drop: Box::new(move |d, e| make_drop(d, e).boxed()),
			window: None,
			positioning: Positioning::Down,
		})
	}

	pub fn new_upward<W: 'static + Widget<T>, DW: Widget<T> + 'static>(
		header: W,
		make_drop: impl Fn(&T, &Env) -> DW + 'static,
		content_height: f64,
	) -> impl Widget<T> {
		header.padding(0.).controller(Dropdown {
			drop: Box::new(move |d, e| make_drop(d, e).boxed()),
			window: None,
			positioning: Positioning::Up(content_height),
		})
	}

	fn show_dropdown(&mut self, data: &mut T, env: &Env, ctx: &mut EventCtx) {
		let widget = (self.drop)(data, env);
		let origin = match self.positioning {
			Positioning::Down => {
				let mut origin = ctx.to_window(Point::new(0., ctx.size().height));
				let insets = ctx.window().content_insets();
				origin.x += insets.x0;
				origin.y += insets.y0;
				origin
			}
			Positioning::Up(content_height) => {
				let mut origin = ctx.to_window(Point::new(0., -content_height));
				let insets = ctx.window().content_insets();
				origin.x += insets.x0;
				origin.y += insets.y0;
				origin
			}
		};

		self.window = Some(
			ctx.new_sub_window(
				WindowConfig::default()
					.set_level(WindowLevel::DropDown(ctx.window().clone()))
					.set_position(origin)
					.window_size_policy(WindowSizePolicy::Content)
					.resizable(false)
					.show_titlebar(false),
				widget.controller(ClosedNotifier {
					parent: ctx.widget_id(),
				}),
				data.clone(),
				env.clone(),
			),
		);
		ctx.set_active(true);
	}
}

struct ClosedNotifier {
	parent: WidgetId,
}

impl<T, W: Widget<T>> Controller<T, W> for ClosedNotifier {
	fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
		if let Event::WindowDisconnected = event {
			ctx.submit_command(DROPDOWN_CLOSED.to(self.parent));
		}
		child.event(ctx, event, data, env);
	}
}

impl<T: Data, W: Widget<T>> Controller<T, W> for Dropdown<T> {
	fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
		match event {
			Event::Command(c) if c.is(DROPDOWN_SHOW) && self.window.is_none() => {
				self.show_dropdown(data, env, ctx);
				ctx.set_handled();
			}
			Event::Notification(n) if n.is(DROPDOWN_SHOW) && self.window.is_none() => {
				self.show_dropdown(data, env, ctx);
				ctx.set_handled();
			}
			Event::Command(cmd) if cmd.is(DROPDOWN_CLOSED) => {
				ctx.set_active(false);
				self.window = None;
				let inner_cmd = cmd.clone().to(Target::Global);
				// send DROP_END to header
				child.event(ctx, &Event::Command(inner_cmd), data, env);
				ctx.set_handled();
			}

			Event::Command(cmd) if cmd.is(DROPDOWN_HIDE) => {
				if let Some(w) = self.window {
					ctx.submit_command(CLOSE_WINDOW.to(w));
				}
				ctx.set_handled();
			}

			Event::Notification(cmd) if cmd.is(DROPDOWN_HIDE) => {
				if let Some(w) = self.window {
					ctx.submit_command(CLOSE_WINDOW.to(w));
				}
				ctx.set_handled();
			}

			// we recieve global mouse downs when widget is_active
			// close on any outside mouse click
			Event::MouseDown(ev) if ctx.is_active() && !ctx.size().to_rect().contains(ev.pos) => {
				if let Some(w) = self.window {
					ctx.submit_command(CLOSE_WINDOW.to(w));
				}
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
		child.lifecycle(ctx, event, data, env)
	}
}
