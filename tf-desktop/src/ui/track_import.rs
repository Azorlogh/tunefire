use druid::{
	im,
	keyboard_types::Key,
	lens,
	widget::{Container, CrossAxisAlignment, Flex, Label, List, Scroll, TextBox, ViewSwitcher},
	Data, Widget, WidgetExt,
};

use crate::{
	command,
	controller::tag_searcher::TagSearch,
	data::ctx::Ctx,
	state::{NewTrack, NewTrackBulk, TagSuggestions, TrackImport},
	theme,
	widget::{
		common::{
			focusable_button::FocusableButton,
			separator::Separator,
			smart_list::{IdentifiedVector, SmartList, ITEM_DELETE},
		},
		controllers::{AutoFocus, ClickAfter, ClickBlocker, ItemDeleter, OnFocus, OnKey},
		tag_edit::TagEdit,
	},
};

pub fn track_import(db: tf_db::Client) -> impl Widget<TrackImport> {
	Container::new(
		Container::new(
			ViewSwitcher::new(
				|data: &TrackImport, _| std::mem::discriminant(data),
				move |_, data, _| match data {
					TrackImport::Single(_) => {
						add_track().lens(enum_lens!(TrackImport::Single)).boxed()
					}
					TrackImport::Bulk(_) => add_bulk(db.clone())
						.lens(enum_lens!(TrackImport::Bulk))
						.boxed(),
				},
			)
			.padding(10.0),
		)
		.controller(ClickBlocker)
		.border(crate::theme::FOREGROUND, 1.0)
		.background(crate::theme::BACKGROUND)
		.padding(100.0),
	)
	.controller(ClickAfter::new(|ctx, _, _| {
		ctx.submit_command(command::UI_TRACK_ADD_CLOSE);
	}))
	.controller(OnKey::new(Key::Escape, |ctx, _, _| {
		ctx.submit_command(command::UI_TRACK_ADD_CLOSE);
	}))
}

pub fn add_bulk(db: tf_db::Client) -> impl Widget<NewTrackBulk> {
	Flex::column()
		.cross_axis_alignment(CrossAxisAlignment::Start)
		.with_child(
			Label::new("Playlist import")
				.with_font(druid::theme::UI_FONT_BOLD)
				.center(),
		)
		.with_default_spacer()
		.with_child(
			Separator::new()
				.with_width(1.0)
				.with_color(theme::BACKGROUND_HIGHLIGHT1),
		)
		.with_default_spacer()
		.with_child(Label::new("Tracks"))
		.with_default_spacer()
		.with_flex_child(
			Scroll::new(
				List::new(|| {
					Flex::row()
						.with_child(track_artists())
						.with_spacer(12.0)
						.with_child(Label::new("-"))
						.with_spacer(12.0)
						.with_child(
							TextBox::new()
								.with_placeholder("Title")
								.controller(OnKey::new(Key::Enter, |ctx, _, _| ctx.focus_next()))
								.lens(NewTrack::title),
						)
				})
				.lens(NewTrackBulk::tracks),
			)
			.vertical(),
			1.0,
		)
		.with_default_spacer()
		.with_child(
			Separator::new()
				.with_width(1.0)
				.with_color(theme::BACKGROUND_HIGHLIGHT1),
		)
		.with_default_spacer()
		.with_child(Label::new("Tags"))
		.with_default_spacer()
		.with_child(
			SmartList::new(|| TagEdit::new(), |data| data.data.0)
				.controller(ItemDeleter::<
					Ctx<TagSuggestions, IdentifiedVector<(String, f32)>>,
					(u128, (String, f32)),
				>::new(|data| data.0))
				.lens(Ctx::make(
					lens::Map::new(
						|s: &NewTrackBulk| s.tag_suggestions.clone(),
						|s, i| {
							if !i.same(&s.tag_suggestions) {
								s.tag_suggestions = i;
							}
						},
					),
					NewTrackBulk::tags,
				))
				.controller(TagSearch::new(&db, NewTrackBulk::tag_suggestions)),
		)
		.with_child(
			FocusableButton::new("+")
				.on_click(|_, data: &mut NewTrackBulk, _| {
					data.tags.push_back((rand::random(), ("".to_owned(), 0.5)));
				})
				.expand_width(),
		)
		.with_default_spacer()
		.with_child(
			Separator::new()
				.with_width(1.0)
				.with_color(theme::BACKGROUND_HIGHLIGHT1),
		)
		.with_default_spacer()
		.with_child(
			FocusableButton::new("Add").on_click(|ctx, data: &mut NewTrackBulk, _| {
				for track in &data.tracks {
					ctx.submit_command(command::TRACK_ADD.with(track.get_track()));
				}
			}),
		)
}

pub fn add_track() -> impl Widget<NewTrack> {
	Flex::column()
		.with_child(Label::new("Add Track").with_font(druid::theme::UI_FONT_BOLD))
		.with_default_spacer()
		.with_child(
			Separator::new()
				.with_width(1.0)
				.with_color(theme::BACKGROUND_HIGHLIGHT1),
		)
		.with_default_spacer()
		.with_child(
			TextBox::new()
				.controller(OnKey::new(Key::Enter, |ctx, _, _| ctx.focus_next()))
				.lens(NewTrack::source)
				.expand_width(),
		)
		.with_child(track_artists())
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
}

fn track_artists() -> impl Widget<NewTrack> {
	Flex::row()
		.with_child(
			List::new(|| {
				TextBox::new()
					.with_placeholder("Artist")
					.lens(lens!((u128, String), 1))
					.controller(AutoFocus)
					.controller(OnKey::new(Key::Enter, |ctx, _, _| ctx.focus_next()))
					.controller(OnFocus::lost(|ctx, data: &mut (u128, String), _| {
						if data.1 == "" {
							ctx.submit_notification(ITEM_DELETE.with(data.0));
						}
					}))
					.fix_width(128.0)
			})
			.horizontal()
			.controller(ItemDeleter::new(|data: &(u128, String)| data.0)),
		)
		.with_child(FocusableButton::new("+").on_click(
			|_, data: &mut im::Vector<(u128, String)>, _| {
				data.push_back((rand::random(), String::new()));
			},
		))
		.lens(NewTrack::artists)
}
