use std::sync::Arc;

use anyhow::Result;
use druid::{
	im, widget::Controller, Env, Event, EventCtx, ExtEventSink, LifeCycle, LifeCycleCtx, Selector,
	Widget, WidgetId,
};
use parking_lot::RwLock;
use tf_plugin::{Plugin, SearchResult};
use tracing::warn;

use crate::State;

pub const SEARCH_TRACK_REQUEST: Selector<String> = Selector::new("plugin.search-track.request");
pub const SEARCH_TRACK_RESULTS: Selector<Vec<SearchResult>> =
	Selector::new("plugin.search.results");

pub struct SearchController;

impl SearchController {
	pub fn new() -> Result<Self> {
		Ok(Self)
	}

	pub fn spawn_search_threads(
		&mut self,
		plugins: &im::Vector<Arc<RwLock<Box<dyn Plugin>>>>,
		sink: ExtEventSink,
		query: &str,
		id: WidgetId,
	) {
		for mut plugin in plugins.iter().filter_map(|p| p.read().get_search_plugin()) {
			let sink = sink.clone();
			let query = query.to_owned();
			std::thread::Builder::new()
				.name(String::from("search task"))
				.spawn(move || match plugin.search(&query) {
					Ok(r) => {
						sink.submit_command(SEARCH_TRACK_RESULTS, r, id).unwrap();
					}
					Err(e) => warn!("{:?}", e),
				})
				.unwrap();
		}
	}
}

impl<W: Widget<State>> Controller<State, W> for SearchController {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut EventCtx,
		event: &Event,
		data: &mut State,
		env: &Env,
	) {
		let handled = match event {
			Event::Command(cmd) => match cmd {
				_ if cmd.is(SEARCH_TRACK_REQUEST) => {
					let q = cmd.get_unchecked::<String>(SEARCH_TRACK_REQUEST);
					data.track_search_results.tracks.clear();

					self.spawn_search_threads(
						&data.plugins,
						ctx.get_external_handle(),
						q,
						ctx.widget_id(),
					);

					druid::Handled::Yes
				}
				_ if cmd.is(SEARCH_TRACK_RESULTS) => {
					let results = cmd.get_unchecked::<Vec<SearchResult>>(SEARCH_TRACK_RESULTS);
					data.track_search_results.tracks.extend(results.clone());
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

	fn lifecycle(
		&mut self,
		child: &mut W,
		ctx: &mut LifeCycleCtx,
		event: &LifeCycle,
		data: &State,
		env: &Env,
	) {
		child.lifecycle(ctx, event, data, env)
	}
}
