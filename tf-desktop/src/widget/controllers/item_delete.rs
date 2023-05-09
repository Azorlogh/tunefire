use std::marker::PhantomData;

use druid::{im, widget::Controller, Data, Env, Selector, Widget};

use crate::data::ctx::{Ctx, CtxEnumerate};

pub const ITEM_DELETE: Selector<usize> = Selector::new("item-delete");

/// Deletes item from list upon receiving ITEM_DELETE
/// The type parameter is just here for ergonomics to help type interference
pub struct ItemDeleter<T>(PhantomData<T>);

impl<T> ItemDeleter<T> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<T, W> Controller<im::Vector<T>, W> for ItemDeleter<im::Vector<T>>
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

impl<T, W, C> Controller<Ctx<C, im::Vector<T>>, W> for ItemDeleter<Ctx<C, im::Vector<T>>>
where
	T: Data,
	C: Data,
	W: Widget<Ctx<C, im::Vector<T>>>,
{
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut Ctx<C, im::Vector<T>>,
		env: &Env,
	) {
		match event {
			druid::Event::Notification(cmd) if cmd.is(ITEM_DELETE) => {
				let idx = cmd.get::<usize>(ITEM_DELETE).unwrap();
				data.data.remove(*idx);
				ctx.set_handled();
			}
			_ => {}
		}
		child.event(ctx, event, data, env);
	}
}

// impl<T, W, C> Controller<CtxEnumerate<C, im::Vector<T>>, W>
// 	for ItemDeleter<CtxEnumerate<C, im::Vector<T>>>
// where
// 	T: Data,
// 	C: Data,
// 	W: Widget<CtxEnumerate<C, im::Vector<T>>>,
// {
// 	fn event(
// 		&mut self,
// 		child: &mut W,
// 		ctx: &mut druid::EventCtx,
// 		event: &druid::Event,
// 		data: &mut CtxEnumerate<C, im::Vector<T>>,
// 		env: &Env,
// 	) {
// 		match event {
// 			druid::Event::Notification(cmd) if cmd.is(ITEM_DELETE) => {
// 				let idx = cmd.get::<usize>(ITEM_DELETE).unwrap();
// 				data.data.remove(*idx);
// 				ctx.set_handled();
// 			}
// 			_ => {}
// 		}
// 		child.event(ctx, event, data, env);
// 	}
// }
