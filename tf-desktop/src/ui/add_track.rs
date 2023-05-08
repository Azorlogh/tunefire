use druid::{
	im,
	keyboard_types::Key,
	widget::{Container, Flex, Label, List, TextBox},
	Widget, WidgetExt,
};

use crate::{
	command,
	data::enumerate::{deenumerate, lens_enumerate},
	state::NewTrack,
	widget::{
		common::focusable_button::FocusableButton,
		controllers::{
			AutoFocus, ClickAfter, ClickBlocker, ItemDeleter, OnFocus, OnKey, ITEM_DELETE,
		},
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
						.with_flex_child(
							List::new(|| {
								TextBox::new()
									.with_placeholder("Artist")
									.lens(deenumerate())
									.controller(AutoFocus)
									.controller(OnKey::new(Key::Enter, |ctx, _, _| {
										ctx.focus_next()
									}))
									.controller(OnFocus::lost(
										|ctx, data: &mut (usize, String), _| {
											if data.1 == "" {
												ctx.submit_notification(ITEM_DELETE.with(data.0));
											}
										},
									))
									.fix_width(128.0)
							})
							.horizontal()
							.lens(lens_enumerate())
							.controller(ItemDeleter),
							1.0,
						)
						.with_child(FocusableButton::new("+").on_click(
							|_, data: &mut im::Vector<String>, _| {
								data.push_back(String::new());
							},
						))
						.expand_width()
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
						ctx.submit_command(command::TRACK_ADD.with(data.get_track()));
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
