use druid::{
	keyboard_types::Key,
	lens,
	widget::{Container, CrossAxisAlignment, Flex, Label, List, TextBox},
	Color, Data, Widget, WidgetExt,
};

use crate::{
	command,
	data::ctx::Ctx,
	state::TrackEdit,
	widget::{common::focusable_button::FocusableButton, controllers::OnKey, tag_edit::TagEdit},
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
		// .border()
		// .env_scope(|env, _| env.set(druid::theme::BORDER_DARK, Color::TRANSPARENT))
		.fix_width(400.0)
		.padding(8.0);
	Container::new(col).background(crate::theme::BACKGROUND_HIGHLIGHT0)
}
