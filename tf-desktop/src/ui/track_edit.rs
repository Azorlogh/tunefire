use druid::{
	im,
	keyboard_types::Key,
	lens,
	widget::{Container, CrossAxisAlignment, Flex, Label, TextBox},
	Data, Widget, WidgetExt,
};

use crate::{
	command,
	controller::tag_searcher::TagSearch,
	data::ctx::Ctx,
	state::{TagSuggestions, TrackEdit},
	widget::{
		common::{
			focusable_button::FocusableButton,
			smart_list::{IdentifiedVector, SmartList, ITEM_DELETE},
		},
		controllers::{ItemDeleter, OnFocus, OnKey},
		tag_edit::TagEdit,
	},
};

pub fn ui(db: &tf_db::Client) -> impl Widget<TrackEdit> {
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
					SmartList::new(
						|| {
							TextBox::new()
								.with_placeholder("Artist")
								.lens(lens!((u128, String), 1))
								.controller(OnKey::new(Key::Enter, |ctx, _, _| ctx.focus_next()))
								.controller(OnFocus::lost(|ctx, data: &mut (u128, String), _| {
									if data.1 == "" {
										ctx.submit_notification(ITEM_DELETE.with(data.0));
									}
								}))
								.fix_width(128.0)
						},
						|data| data.0,
					)
					.horizontal()
					.controller(ItemDeleter::new(|d: &(u128, String)| d.0)),
					1.0,
				)
				.with_child(FocusableButton::new("+").on_click(
					|_, data: &mut im::Vector<(u128, String)>, _| {
						data.push_back((rand::random(), String::new()));
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
		.with_child(
			SmartList::new(|| TagEdit::new(), |data| data.data.0)
				.controller(ItemDeleter::<
					Ctx<TagSuggestions, IdentifiedVector<(String, f32)>>,
					(u128, (String, f32)),
				>::new(|data| data.0))
				.lens(Ctx::make(
					lens::Map::new(
						|s: &TrackEdit| s.tag_suggestions.clone(),
						|s, i| {
							if !i.same(&s.tag_suggestions) {
								s.tag_suggestions = i;
							}
						},
					),
					TrackEdit::tags,
				))
				.controller(TagSearch::new(&db, TrackEdit::tag_suggestions)),
		)
		.with_child(
			FocusableButton::new("+").on_click(|_, data: &mut TrackEdit, _| {
				data.tags.push_back((rand::random(), ("".to_owned(), 0.5)));
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
