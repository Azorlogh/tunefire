use std::{rc::Rc, time::Duration};

use druid::{
	widget::{EnvScope, Flex, Label, List, Maybe, SizedBox, Tabs, TabsTransition},
	Insets, Widget, WidgetExt,
};
use tf_db::Track;

use crate::{theme, State};

pub fn ui() -> impl Widget<State> {
	Tabs::new()
		.with_transition(TabsTransition::Slide(
			Duration::from_millis(100).as_nanos() as u64
		))
		.with_tab("Queue", {
			Flex::column()
				.with_child(Maybe::new(track_ui, || SizedBox::empty()).lens(State::current_track))
				.with_flex_child(List::new(track_ui).lens(State::queue), 1.0)
		})
		.with_tab("History", List::new(track_ui).lens(State::history))
}

fn track_ui() -> impl Widget<Rc<Track>> {
	Flex::column()
		.with_child(
			Label::new(|track: &Rc<Track>, _: &_| track.title.to_owned())
				.with_text_size(16.0)
				.fix_height(18.0)
				.expand_width(),
		)
		.with_child(EnvScope::new(
			|env, _| env.set(druid::theme::TEXT_COLOR, env.get(theme::FOREGROUND_DIM)),
			Label::new(|item: &Rc<Track>, _: &_| item.artist.to_owned())
				.with_text_size(13.0)
				.fix_height(10.0)
				.expand_width(),
		))
		.padding(Insets::new(0.0, 0.0, 0.0, 8.0))
}
