use druid::{widget::Controller, Env, Event, EventCtx, LifeCycle, LifeCycleCtx, Selector, Widget};
use tf_plugin::ImportedItem;
use tracing::warn;

use crate::{command, state::NewTrack, State};

pub const IMPORT_REQUEST: Selector<String> = Selector::new("plugin.import.request");

pub struct ImportController;

impl<W: Widget<State>> Controller<State, W> for ImportController {
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
				_ if cmd.is(IMPORT_REQUEST) => {
					let url = cmd.get_unchecked::<String>(IMPORT_REQUEST);
					if let Ok(url) = url.parse() {
						for mut plugin in data
							.plugins
							.iter()
							.filter_map(|p| p.read().get_import_plugin())
						{
							if let Some(res) = plugin.import(&url) {
								println!("RES: {:?}", res);
								match res {
									Ok(item) => match item {
										ImportedItem::Track(track) => {
											println!("overwriting newtraack");
											ctx.submit_command(command::UI_TRACK_ADD_OPEN.with(
												NewTrack {
													source: url.to_string(),
													title: track.title,
													artists: track.artists,
												},
											));
										}
										_ => todo!(),
									},
									Err(e) => warn!("failed to import {url}: {e}"),
								}
							}
						}
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
