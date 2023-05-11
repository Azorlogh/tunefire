use druid::{
	im,
	keyboard_types::Key,
	lens,
	widget::{Container, Flex, Label, List, Scroll, TextBox, ViewSwitcher},
	Widget, WidgetExt,
};

use crate::{
	command,
	state::{NewTrack, NewTrackBulk, TrackImport},
	widget::{
		common::{focusable_button::FocusableButton, smart_list::ITEM_DELETE},
		controllers::{AutoFocus, ClickAfter, ClickBlocker, ItemDeleter, OnFocus, OnKey},
	},
};

pub fn track_import() -> impl Widget<TrackImport> {
	Container::new(
		Container::new(
			ViewSwitcher::new(
				|data: &TrackImport, _| std::mem::discriminant(data),
				|_, data, _| match data {
					TrackImport::Single(_) => {
						add_track().lens(enum_lens!(TrackImport::Single)).boxed()
					}
					TrackImport::Bulk(_) => add_bulk().lens(enum_lens!(TrackImport::Bulk)).boxed(),
				},
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

pub fn add_bulk() -> impl Widget<NewTrackBulk> {
	Flex::column()
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
		.with_child(Label::new("optional tag"))
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
