use std::marker::PhantomData;

use druid::{im, widget::Controller, Data, Env, Event, EventCtx, Lens, Selector, Widget};

use crate::state::TagSuggestions;

pub const TAG_SEARCH: Selector<String> = Selector::new("tag-search.search");

pub struct TagSearch<T, L: Lens<T, TagSuggestions>> {
	db: tf_db::Client,
	lens: L,
	_pd: PhantomData<T>,
}

impl<T, L: Lens<T, TagSuggestions>> TagSearch<T, L> {
	pub fn new(db: &tf_db::Client, lens: L) -> Self {
		Self {
			db: db.clone(),
			lens,
			_pd: PhantomData,
		}
	}
}

impl<T: Data, L: Lens<T, TagSuggestions>, W: Widget<T>> Controller<T, W> for TagSearch<T, L> {
	fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
		let handled = match event {
			Event::Notification(cmd) => match cmd {
				_ if cmd.is(TAG_SEARCH) => {
					let q = cmd.get::<String>(TAG_SEARCH).unwrap();
					if q != "" {
						let results = self.db.search_tag(q, 3).unwrap();
						self.lens.with_mut(data, |suggestions| {
							suggestions.tags =
								im::Vector::from_iter(results.into_iter().map(|(tag, _)| tag));
						});
					}
					druid::Handled::Yes
				}
				_ => druid::Handled::No,
			},
			_ => druid::Handled::No,
		};

		if handled.is_handled() {
			ctx.set_handled();
		}

		child.event(ctx, event, data, env);
	}
}
