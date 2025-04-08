use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Result};

#[macro_use]
mod util;

mod command;
mod state;
use crossbeam_channel::Receiver;
// mod ui;
use iced::alignment::Vertical::Top;
use iced::event::listen_raw;
// pub use state::State;
// use tf_gui::data;
use iced::widget::{
	button, center, column, container, horizontal_space, pick_list, row, scrollable, slider, text,
	text_editor, text_input, toggler, tooltip, vertical_space, Column, Scrollable, Text, Themer,
};
use iced::{keyboard, Center, Element, Fill, Font, Size, Subscription, Task, Theme};
use parking_lot::RwLock;
use tf_db::Track;
use tf_player::player::{Controller, Event};
use tf_player::TrackSource;
use tracing::warn;
use tracing_subscriber::EnvFilter;

use tf_plugin::Plugin;
use url::Url;
use uuid::Uuid;
// pub mod widget;
// pub mod theme;

mod delegate;

mod media_controls;

mod controller;

const PADDING: u16 = 5;

fn main() -> iced::Result {
	// init tracing
	// use tracing_subscriber::prelude::*;
	// let fmt_layer = tracing_subscriber::fmt::layer()
	// 	.without_time()
	// 	.with_target(true)
	// 	.with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
	// 		metadata.target().starts_with("tf");
	// 		true
	// 	}));
	// tracing_subscriber::registry()
	// 	.with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")))
	// 	.with(fmt_layer)
	// 	.init();

	// start app
	let db = connect_to_db().expect("Could not connect to db");

	iced::application("Tunefire", Tunefire::update, Tunefire::view)
		.window_size(Size::new(1080.0, 720.0))
		.subscription(Tunefire::subscription)
		.theme(Tunefire::theme)
		.run_with(|| Tunefire::new(db))
}

fn connect_to_db() -> Result<tf_db::Client> {
	let dirs = directories::ProjectDirs::from("", "Azorlogh", "tunefire")
		.expect("failed to get data directory");
	std::fs::create_dir_all(dirs.data_dir())?;
	let db_path = dirs.data_dir().join("db.slab");
	tf_db::Client::new(db_path)
}

#[derive(Debug, Default, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
enum SearchSource {
	#[default]
	All,
	Local,
	Soundcloud,
	Youtube,
}

impl std::fmt::Display for SearchSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			SearchSource::All => "All",
			SearchSource::Local => "Local",
			SearchSource::Soundcloud => "SoundCloud",
			SearchSource::Youtube => "YouTube",
		})
	}
}

struct Tunefire {
	current_idx: usize,
	// current_track: Option<Track>,
	db: tf_db::Client,
	player_controller: Controller,
	player_event: Receiver<Event>,
	plugins: Vec<Arc<RwLock<Box<dyn Plugin>>>>,
	search_query: String,
	search_source: SearchSource,
	tag_filter: String,
	theme: Theme,
	track_list: Vec<Track>, // TODO use playlists instead
	volume: f32,
	queue: Vec<Track>,
	playlists: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
	AddPlaylist(String),
	DeletePlaylist(String),
	AddTrackToQueue(TrackSource),
	QueryTag,
	QueryTagChange(String),
	Randomizer,
	RequestPlayTrack(Track),
	Search,
	SearchChange(String),
	SourceChange(SearchSource),
	Tags,
	ChangeVolume(f32),
	PlayPause,
	Next,
	Previous,
	DeleteTrack(Track),
}

impl Tunefire {
	fn new(db: tf_db::Client) -> (Self, Task<Message>) {
		let track_list = db
			.to_owned()
			.iter_tracks()
			.map(|t| t.unwrap().1.to_owned())
			.collect();
		let playlists = db
			.to_owned()
			.iter_playlists()
			.map(|p| p.unwrap().1.to_owned())
			.collect();
		let (player_controller, player_event) = tf_player::player::Player::spawn().unwrap();

		let mut plugins: Vec<Box<dyn Plugin>> = vec![];
		#[cfg(feature = "local")]
		plugins.push(Box::new(tf_plugin_local::Local));
		// #[cfg(feature = "soundcloud")]
		// plugins.push(Box::new(tf_plugin_soundcloud::Soundcloud::new().unwrap()));
		// #[cfg(feature = "youtube")]
		// plugins.push(Box::new(tf_plugin_youtube::Youtube::new().unwrap()));

		(
			Self {
				// current_track: Option::None,
				current_idx: 0,
				db,
				player_controller,
				player_event,
				plugins: plugins
					.into_iter()
					.map(|p| Arc::new(RwLock::new(p)))
					.collect(),
				search_query: String::from(""),
				search_source: SearchSource::All,
				tag_filter: String::from(""),
				theme: Theme::Dark,
				track_list,
				volume: 50.0,
				queue: Vec::new(),
				playlists,
			},
			Task::none(),
		)
	}

	fn add_playlist(&mut self, name: &str) -> Result<Uuid> {
		let id = self.db.add_playlist(name)?;
		if !self.playlists.iter().any(|v| v == name) {
			self.playlists.push(name.to_string());
		}

		Ok(id)
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::DeletePlaylist(name) => {
				let pid = self.db.get_playlist_by_name(&name).unwrap();
				let _ = self.db.delete_playlist(pid);
				if let Some(index) = self.playlists.iter().position(|value| *value == name) {
					self.playlists.swap_remove(index);
				}
				Task::none()
			}
			Message::AddPlaylist(name) => {
				self.add_playlist(&name)
					.expect("Could not add playlist : {name}");
				Task::none()
			}
			Message::Randomizer => {
				println!("Random music player");

				Task::none()
			}
			Message::Tags => {
				println!("Show tag window");

				Task::none()
			}
			Message::QueryTag => {
				self.filter_tracks_by_tag_expression();

				Task::none()
			}
			Message::QueryTagChange(query) => {
				self.tag_filter = query;

				Task::none()
			}
			Message::Search => {
				self.search_track_from_source();

				Task::none()
			}
			Message::SearchChange(query) => {
				self.search_query = query;

				Task::none()
			}
			Message::SourceChange(search_source) => {
				self.search_source = search_source;

				Task::none()
			}
			Message::RequestPlayTrack(track) => {
				let current_track = &self.queue;
				println!("Current Queue : {:?}", current_track);
				println!("Requesting : {:?}", track);
				// match current_track {
				// 	Some(_) => {
				// 		self.queue.push(track.clone());
				// 	}
				// 	None => current_track = Some(track.clone()),
				// };

				// self.request_track_audio_source(&track)
				Task::none()
			}
			Message::AddTrackToQueue(source) => {
				self.player_controller.queue_track(source).unwrap();

				Task::none()
			}
			Message::ChangeVolume(val) => {
				self.volume = val;

				let _ = self.player_controller.set_volume(self.volume / 100.0);

				Task::none()
			}
			Message::PlayPause => {
				let _ = self.player_controller.play_pause();

				Task::none()
			}
			Message::Next => {
				println!("{:?}", self.queue);
				println!("{:?}", self.current_idx);
				// match self.queue.pop() {
				// 	Some(t) => {
				// 		self.current_track = Some(t.clone());
				// 		self.player_controller.skip().unwrap();
				// 		self.player_controller.play().unwrap();
				// 		return self.request_track_audio_source(&t);
				// 	}
				// 	None => {}
				// }

				Task::none()
			}
			Message::Previous => {
				// println!("{:?}", self.history);
				// match self.history.pop() {
				// 	Some(t) => {
				// 		self.queue.push(self.current_track.clone().unwrap());
				// 		self.current_track = Some(t.clone());
				// 		self.player_controller.previous().unwrap();
				// 		self.player_controller.play().unwrap();
				// 		return self.request_track_audio_source(&t);
				// 	}
				// 	None => {}
				// }

				Task::none()
			}
			Message::DeleteTrack(track) => {
				println!("Deleting : {:?}", track);
				if let Some(index) = self
					.track_list
					.iter()
					.position(|value| *value.source == track.source)
				{
					self.track_list.swap_remove(index);
				}
				Task::none()
			}
		}
	}

	fn request_track_audio_source(&self, track: &Track) -> Task<Message> {
		let url = Url::parse(&track.source).unwrap();
		let plugins = self.plugins.clone();
		Task::perform(
			async move {
				if let Some(result) = plugins
					.iter()
					.filter_map(|p| p.read().get_source_plugin())
					.find_map(|p| p.handle_url(&url))
				{
					match result {
						Ok(source) => Some(source),
						Err(e) => {
							warn!("error while handling track {url:?}: {e}");
							Option::None
						}
					}
				} else {
					warn!("no plugin could handle the track: {url:?}");
					Option::None
				}
			},
			|res| res,
		)
		.and_then(|res| Task::done(Message::AddTrackToQueue(res)))
	}

	fn view(&self) -> Element<Message> {
		// sidebar
		let width = 120.0;

		let playlists = container(column(self.playlists.iter().map(|t| {
			row![
				text(t),
				button("DELETE").on_press(Message::DeletePlaylist(t.to_owned()))
			]
			.into()
		})));

		let sidebar = column![
			button("Randomizer")
				.width(width)
				.on_press(Message::Randomizer),
			button("Tags").width(width).on_press(Message::Tags),
			button("Add playlist")
				.width(width)
				.on_press(Message::AddPlaylist(
					String::from_str("Default").unwrap() // TODO
				)),
			playlists,
		]
		.align_x(Center)
		.spacing(10);

		// tag filter bar
		let tag_filter_bar = text_input("tag filter", &self.tag_filter)
			.align_x(Center)
			.on_input(Message::QueryTagChange)
			.on_submit(Message::QueryTag);

		// track list
		let content = container(
			column(self.track_list.iter().map(|t| {
				row![
					button("PLAY").on_press(Message::RequestPlayTrack(t.to_owned())),
					text(t.artists.join(", ")),
					text(" - "),
					text(t.title.to_owned()),
					button("DELETE").on_press(Message::DeleteTrack(t.to_owned())),
				]
				.into()
			}))
			.spacing(20.0),
		)
		.center_x(Fill);

		let track_list = scrollable(content)
			.direction(scrollable::Direction::Vertical(
				scrollable::Scrollbar::new().width(1).scroller_width(10),
			))
			.width(Fill)
			.height(Fill);

		// source_selector
		// TODO
		let source_selector = pick_list(
			[
				SearchSource::Local,
				SearchSource::Soundcloud,
				SearchSource::Youtube,
				SearchSource::All,
			],
			Some(self.search_source),
			|source| Message::SourceChange(source),
		);

		// search bar
		// TODO
		let search_bar = text_input("Search", &self.search_query)
			.align_x(Center)
			.on_input(Message::SearchChange)
			.on_submit(Message::Search);

		// current track
		// TODO
		// let media_bar = match &self.current_track {
		// 	Some(t) => row![
		// 		row![
		// 			text("artwork"),
		// 			column![
		// 				text(format!("{}", t.title)),
		// 				text(format!("{}", t.artists.join(", ")))
		// 			]
		// 		]
		// 		.spacing(4.0),
		// 		horizontal_space(),
		// 		column![
		// 			row![
		// 				button("shuffle"),
		// 				button("previous").on_press(Message::Previous),
		// 				button("play_pause").on_press(Message::PlayPause),
		// 				button("next").on_press(Message::Next),
		// 				button("replay")
		// 			]
		// 			.spacing(4.0),
		// 			text("track timer bar")
		// 		],
		// 		horizontal_space(),
		// 		column![row![
		// 			text("volume"),
		// 			slider(0.0..=100.0, self.volume, Message::ChangeVolume)
		// 		]
		// 		.spacing(4.0)],
		// 	]
		// 	.align_y(Center),
		// 	None => row![text("No track.")],
		// };

		container(column![
			row![sidebar, column![tag_filter_bar, track_list]],
			row![source_selector, search_bar],
			// media_bar,
		])
		.padding([PADDING, PADDING])
		.into()
	}

	fn subscription(&self) -> Subscription<Message> {
		use keyboard::key;

		keyboard::on_key_release(|key, _modifiers| match key {
			key::Key::Named(named) => None,
			key::Key::Character(_) => None,
			key::Key::Unidentified => None,
		})
	}

	fn theme(&self) -> Theme {
		self.theme.clone()
	}

	fn search_track_from_source(&self) {
		println!(
			"Search '{:?}' from source '{}'",
			self.search_query,
			self.search_source.to_string()
		)
	}

	fn filter_tracks_by_tag_expression(&self) {
		println!(
			"Filter tracks with expression '{}'",
			self.tag_filter.to_string()
		)
	}
}
