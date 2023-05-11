use std::marker::PhantomData;

use druid::{im, widget::Controller, Data, Env, Widget};

use crate::{
	data::ctx::Ctx,
	widget::common::smart_list::{ItemId, ITEM_DELETE},
};

/// Deletes item from list upon receiving ITEM_DELETE
pub struct ItemDeleter<T, I> {
	get_id: Box<dyn Fn(&I) -> ItemId>,
	pd: PhantomData<T>,
}

impl<T, I> ItemDeleter<T, I> {
	pub fn new(get_id: impl Fn(&I) -> ItemId + 'static) -> Self {
		Self {
			get_id: Box::new(move |data| get_id(data)),
			pd: PhantomData,
		}
	}
}

impl<T, W> Controller<im::Vector<T>, W> for ItemDeleter<im::Vector<T>, T>
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
				let id = cmd.get::<u128>(ITEM_DELETE).unwrap();
				let idx = data
					.iter()
					.position(|item| (self.get_id)(item) == *id)
					.unwrap();
				data.remove(idx);
				ctx.set_handled();
			}
			_ => {}
		}
		child.event(ctx, event, data, env);
	}
}

impl<T, W, C> Controller<Ctx<C, im::Vector<T>>, W> for ItemDeleter<Ctx<C, im::Vector<T>>, T>
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
				let id = cmd.get::<u128>(ITEM_DELETE).unwrap();
				let idx = data
					.data
					.iter()
					.position(|item| (self.get_id)(item) == *id)
					.unwrap();
				data.data.remove(idx);
				ctx.set_handled();
			}
			_ => {}
		}
		child.event(ctx, event, data, env);
	}
}
