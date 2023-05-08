use druid::{
	im,
	keyboard_types::Key,
	widget::{Button, Container, Flex, Label, List, TextBox},
	Widget, WidgetExt,
};

use crate::{
	command,
	state::NewTrack,
	widget::{
		common::focusable_button::FocusableButton,
		controllers::{AutoFocus, ClickAfter, ClickBlocker, OnKey},
	},
};

pub fn add_track() -> impl Widget<NewTrack> {
	Container::new(
		Container::new(
			Flex::column()
				.with_child(Label::new("Add Track").with_font(druid::theme::UI_FONT_BOLD))
				.with_default_spacer()
				.with_child(
					TextBox::new()
						.controller(OnKey::new(Key::Enter, |ctx, _, _| ctx.focus_next()))
						.lens(NewTrack::source)
						.expand_width(),
				)
				.with_child(
					Flex::row()
						.with_child(List::new(|| {
							TextBox::new()
								.with_placeholder("Artist")
								.controller(AutoFocus)
								.controller(OnKey::new(Key::Enter, |ctx, _, _| ctx.focus_next()))
								.expand_width()
						}))
						.with_child(Button::new("+").on_click(
							|_, data: &mut im::Vector<String>, _| {
								data.push_back(String::new());
							},
						))
						.lens(NewTrack::artists),
				)
				.with_child(
					TextBox::new()
						.with_placeholder("Title")
						.controller(OnKey::new(Key::Enter, |ctx, _, _| ctx.focus_next()))
						.lens(NewTrack::title)
						.expand_width(),
				)
				.with_default_spacer()
				.with_child(
					FocusableButton::new("Add").on_click(|ctx, data: &mut NewTrack, _| {
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
	.controller(OnKey::new(Key::Escape, |ctx, _, _| {
		ctx.submit_command(command::UI_TRACK_ADD_CLOSE);
	}))
}
