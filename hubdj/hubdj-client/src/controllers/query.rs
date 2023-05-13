use std::sync::Arc;

use druid::{im, widget::Controller, Env, Event, EventCtx, Selector, Widget};
use tracing::warn;

use crate::state::{self, Tracklist};

pub const QUERY_RUN: Selector<String> = Selector::new("query.run");

pub struct QueryController {
	db: tf_db::Client,
}

impl QueryController {
	pub fn new(db: &tf_db::Client) -> Self {
		Self { db: db.clone() }
	}
}

type WData = Tracklist;

impl<W: Widget<WData>> Controller<WData, W> for QueryController {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut EventCtx,
		event: &Event,
		data: &mut WData,
		env: &Env,
	) {
		let handled = match event {
			Event::Notification(cmd) => match cmd {
				_ if cmd.is(QUERY_RUN) => {
					match cmd
						.get::<String>(QUERY_RUN)
						.unwrap()
						.parse::<tf_db::Filter>()
						.and_then(|f| self.db.list_filtered(&f))
					{
						Ok(results) => {
							data.tracks =
								im::Vector::from_iter(results.into_iter().map(|(_, track)| {
									state::Track {
										artists: track.artists.join(", "),
										source: Arc::from(track.source.clone()),
										title: Arc::from(track.title.clone()),
									}
								}));
						}
						Err(e) => warn!("failed to query: {e}"),
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
