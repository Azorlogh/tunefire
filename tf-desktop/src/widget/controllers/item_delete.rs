use druid::{im, widget::Controller, Data, Env, Selector, Widget};

pub const ITEM_DELETE: Selector<usize> = Selector::new("item-delete");

pub struct ItemDeleter;

impl<T, W> Controller<im::Vector<T>, W> for ItemDeleter
where
	T: Data,
	W: Widget<im::Vector<T>>,
{
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut im::Vector<T>,
		env: &Env,
	) {
		match event {
			druid::Event::Notification(cmd) if cmd.is(ITEM_DELETE) => {
				let idx = cmd.get::<usize>(ITEM_DELETE).unwrap();
				data.remove(*idx);
				ctx.set_handled();
			}
			_ => {}
		}
		child.event(ctx, event, data, env);
	}
}
