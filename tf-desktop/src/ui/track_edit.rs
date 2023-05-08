use druid::{
	im,
	keyboard_types::Key,
	lens,
	widget::{Container, CrossAxisAlignment, Flex, Label, List, TextBox},
	Data, Widget, WidgetExt,
};

use crate::{
	command,
	data::{
		ctx::Ctx,
		enumerate::{deenumerate, lens_enumerate},
	},
	state::TrackEdit,
	widget::{
		common::focusable_button::FocusableButton,
		controllers::{ItemDeleter, OnFocus, OnKey, ITEM_DELETE},
		tag_edit::TagEdit,
	},
};

pub fn ui() -> impl Widget<TrackEdit> {
	let col = Flex::column()
		.cross_axis_alignment(CrossAxisAlignment::Fill)
		.with_child(
			Flex::row()
				.with_child(Label::new("title"))
				.with_child(TextBox::new().lens(TrackEdit::title)),
		)
		.with_default_spacer()
		.with_child(
			Flex::row()
				.with_flex_child(
					List::new(|| {
						TextBox::new()
							.with_placeholder("Artist")
							.lens(deenumerate())
							.controller(OnKey::new(Key::Enter, |ctx, _, _| ctx.focus_next()))
							.controller(OnFocus::lost(|ctx, data: &mut (usize, String), _| {
								if data.1 == "" {
									ctx.submit_notification(ITEM_DELETE.with(data.0));
								}
							}))
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
				.lens(TrackEdit::artists),
		)
		.with_default_spacer()
		.with_child(
			Flex::row()
				.with_child(Label::new("source"))
				.with_child(TextBox::new().lens(TrackEdit::source)),
		)
		.with_default_spacer()
		.with_child(List::new(|| TagEdit::new()).lens(Ctx::make(
			lens::Map::new(
				|s: &TrackEdit| s.tag_suggestions.clone(),
				|s, i| {
					if !i.same(&s.tag_suggestions) {
						s.tag_suggestions = i;
					}
				},
			),
			TrackEdit::tags,
		)))
		.with_child(
			FocusableButton::new("+").on_click(|_, data: &mut TrackEdit, _| {
				data.tags.push_back(("".to_owned(), 0.5));
			}),
		)
		.with_flex_spacer(1.0)
		.with_child(
			FocusableButton::new("CLOSE").on_click(|ctx, _: &mut TrackEdit, _| {
				ctx.submit_command(command::UI_TRACK_EDIT_CLOSE)
			}),
		)
		.fix_width(400.0)
		.padding(8.0);
	Container::new(col).background(crate::theme::BACKGROUND_HIGHLIGHT0)
}
