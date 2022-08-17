use druid::{
	widget::{Button, Container, Flex, Label, TextBox},
	Widget, WidgetExt,
};

use crate::{
	command,
	state::NewTrack,
	widget::controllers::{AutoFocus, ClickAfter, ClickBlocker, Enter, Focusable},
};

pub fn add_track() -> impl Widget<NewTrack> {
	Container::new(
		Container::new(
			Flex::column()
				.with_child(Label::new("Add Track").with_font(druid::theme::UI_FONT_BOLD))
				.with_default_spacer()
				.with_child(
					TextBox::new()
						.controller(Enter::new(|ctx, _, _| ctx.focus_next()))
						.lens(NewTrack::source)
						.expand_width(),
				)
				.with_child(
					TextBox::new()
						.with_placeholder("Artist")
						.controller(AutoFocus)
						.controller(Enter::new(|ctx, _, _| ctx.focus_next()))
						.lens(NewTrack::artist)
						.expand_width(),
				)
				.with_child(
					TextBox::new()
						.with_placeholder("Title")
						.controller(Enter::new(|ctx, _, _| ctx.focus_next()))
						.lens(NewTrack::title)
						.expand_width(),
				)
				.with_default_spacer()
				.with_child(
					Button::new("Add")
						.controller(Focusable)
						.controller(Enter::new(|ctx, data: &mut NewTrack, _| {
							ctx.submit_command(command::TRACK_ADD.with(data.clone()));
						}))
						.on_click(|ctx, data: &mut NewTrack, _| {
							ctx.submit_command(command::TRACK_ADD.with(data.clone()));
						}),
				)
				.padding(10.0),
		)
		.controller(ClickBlocker)
		.border(crate::theme::FOREGROUND, 1.0)
		.background(crate::theme::BACKGROUND)
		.padding(200.0),
	)
	.controller(ClickAfter::new(|ctx, _, _| {
		ctx.submit_command(command::UI_TRACK_ADD_CLOSE);
	}))
}
