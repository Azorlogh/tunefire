use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display};
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
	text_editor, text_input, toggler, tooltip, vertical_space, Column, Row, Scrollable, Text,
	Themer,
};
use iced::{keyboard, padding, Center, Element, Fill, Font, Size, Subscription, Task, Theme};
use parking_lot::RwLock;
use tf_db::{Playlist, PlaylistId, Track};
use tf_player::player::{Controller, Event};
use tf_player::TrackSource;
use tracing::{warn, Instrument};
use tracing_subscriber::EnvFilter;

use tf_plugin::Plugin;
use url::Url;
use uuid::Uuid;
// pub mod widget;
// pub mod theme;

use tf_db::TrackId;
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
		.window_size(Size::new(1270.0, 720.0))
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
	Soundcloud,
	Youtube,
}

impl std::fmt::Display for SearchSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			SearchSource::All => "All",
			SearchSource::Soundcloud => "SoundCloud",
			SearchSource::Youtube => "YouTube",
		})
	}
}

#[derive(Debug)]
pub enum ViewType {
	Tracks,
	Favorites,
	Playlist(PlaylistId),
	Search,
}

// History : last played song, allow duplicate
// TODO
// Queue : Songs to be played, ordered (can be randomized)
// When we request to play a song, add everything from the playlist to the queue
// When/If randomize is selected, randomize the queue after putting everything on it.
// If "Keep listening" is selected, when the queue is finished, make one again with the same playlist.

struct Tunefire {
	db: tf_db::Client,
	player_controller: Controller,
	player_event: Receiver<Event>,
	plugins: Vec<Arc<RwLock<Box<dyn Plugin>>>>,
	search_query: String,
	search_source: SearchSource,
	tag_filter: String,
	theme: Theme,
	volume: f32,
	queue: VecDeque<TrackId>,
	track_list: Vec<TrackId>,
	playlists: Vec<PlaylistId>,
	current_view: ViewType,
	current_track_id: Option<TrackId>,
	add_playlist_name: String,
	history: Vec<TrackId>, // TODO max size ? https://docs.rs/bounded-vec-deque/0.1.0/bounded_vec_deque/struct.BoundedVecDeque.html
}

#[derive(Debug, Clone)]
enum Message {
	// Playlist
	AddPlaylistNameChange(String),
	AddPlaylist,
	DeletePlaylist(PlaylistId),
	ShowPlaylist(PlaylistId),
	AddTrackToPlaylist(TrackId, String),
	// Tracks
	AddTrackToTracks(Track),
	ShowTrackList,
	ImportLocalTracks,
	DeleteAllTracks,
	AddTrackToControllerQueue(TrackSource),
	RequestPlayTrack(TrackId),
	ChangeVolume(f32),
	PlayPause,
	Next,
	Previous,
	DeleteTrack(TrackId),
	RemoveTrackFromPlaylist(PlaylistId, TrackId),
	// Tags
	ShowTags,
	QueryTag,
	QueryTagChange(String),
	// Pages
	// Search
	Search,
	SearchChange(String),
	SourceChange(SearchSource),
}

impl Tunefire {
	fn new(db: tf_db::Client) -> (Self, Task<Message>) {
		let track_list = db
			.to_owned()
			.iter_tracks()
			.map(|t| t.unwrap().0.to_owned())
			.collect();

		let playlists = db
			.to_owned()
			.iter_playlists()
			.map(|p| (p.unwrap().0.to_owned()))
			.collect();

		let (player_controller, player_event) = tf_player::player::Player::spawn().unwrap();

		let mut plugins: Vec<Box<dyn Plugin>> = vec![];
		#[cfg(feature = "local")]
		plugins.push(Box::new(tf_plugin_local::Local));
		#[cfg(feature = "soundcloud")]
		plugins.push(Box::new(tf_plugin_soundcloud::Soundcloud::new().unwrap()));
		// #[cfg(feature = "youtube")]
		// plugins.push(Box::new(tf_plugin_youtube::Youtube::new().unwrap()));

		let volume = 10.0;
		(
			Self {
				// current_track: Option::None,
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
				volume,
				track_list,
				queue: VecDeque::new(),
				playlists,
				current_track_id: Option::None,
				current_view: ViewType::Tracks,
				add_playlist_name: String::new(),
				history: Vec::new(),
			},
			Task::done(Message::ChangeVolume(volume)),
		)
	}

	fn get_local_tracks(&mut self) -> Result<Vec<Track>> {
		match rfd::FileDialog::new()
			.add_filter("music", &["mp3", "m4a"])
			.pick_files()
		{
			Some(f) => {
				let tracks: Vec<Track> = f
					.iter()
					.map(|p| Track {
						source: "file://".to_string() + p.to_str().unwrap(),
						artists: vec!["Test".to_string()],
						title: p.file_name().unwrap().to_string_lossy().to_string(),
						tags: HashMap::new(),
					})
					.collect();

				Ok(tracks)
			}
			None => Ok(Vec::new()),
		}
	}

	fn update_playlists(&mut self) {
		self.playlists = self
			.db
			.to_owned()
			.iter_playlists()
			.map(|p| (p.unwrap().0.to_owned()))
			.collect();
	}

	fn update_tracks(&mut self) {
		self.track_list = self
			.db
			.to_owned()
			.iter_tracks()
			.map(|t| t.unwrap().0.to_owned())
			.collect();
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::DeletePlaylist(id) => {
				let _ = self.db.delete_playlist(id);
				self.update_playlists();

				match &self.current_view {
					ViewType::Playlist(pid) => {
						if pid == &id {
							self.current_view = ViewType::Tracks;
						}
					}
					_ => {}
				}
				Task::none()
			}
			Message::AddPlaylist => {
				if !self.add_playlist_name.is_empty() {
					let playlist = Playlist {
						name: self.add_playlist_name.to_owned(),
						track_ids: Vec::new(),
					};
					let pid = self
						.db
						.add_playlist(&playlist)
						.expect("Could not add playslit to db.");
					self.update_playlists();
					self.current_view = ViewType::Playlist(pid);
				}
				self.add_playlist_name = String::new();
				Task::none()
			}
			Message::ShowPlaylist(id) => {
				println!("Showing playlist view for : {}", id);
				let _ = self.db.get_playlist(id).expect("Could not get playlist.");
				self.current_view = ViewType::Playlist(id);
				Task::none()
			}
			Message::AddPlaylistNameChange(name) => {
				self.add_playlist_name = name;
				Task::none()
			}
			Message::AddTrackToPlaylist(track_id, playlist_name) => {
				println!("{}", playlist_name);
				// println!("Adding track {:?} to playlist {:?}", track_id, playlist_id);

				Task::none()
			}
			Message::ShowTags => {
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
				self.current_view = ViewType::Search;
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
			Message::RequestPlayTrack(track_id) => {
				match self.history.len() {
					0 => self.history.push(track_id),
					n => {
						if track_id != self.history[n - 1] {
							self.history.push(track_id);
						}
					}
				};

				match self.queue.len() {
					0 => self.current_track_id = Some(track_id),
					_ => {}
				}

				self.queue.push_front(track_id);
				self.request_track_audio_source(&track_id)
			}
			Message::AddTrackToControllerQueue(source) => {
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
				let _ = self.player_controller.skip();
				self.queue.pop_back();
				match self.queue.len() {
					0 => self.current_track_id = Option::None,
					n => self.current_track_id = Some(self.queue[n - 1]),
				}

				Task::none()
			}
			Message::Previous => {
				// TODO, how for controller ?
				// match self.history.len() {
				// 	0 => {}
				// 	n => {
				// 		let t = self.history[0];
				// 		self.queue.push_front(t);
				// 		self.current_track_id = Some(t);
				// 	}
				// }
				println!("Asking for previous");

				Task::none()
			}
			Message::DeleteTrack(track_id) => {
				println!("Deleting : {:?}", track_id);
				let _ = self.db.delete_track(track_id);
				self.update_tracks();
				Task::none()
			}
			Message::RemoveTrackFromPlaylist(playlist_id, track_id) => {
				println!("Deleting : {:?} from {:?}", track_id, playlist_id);

				let _ = self.db.delete_track_from_playlist(playlist_id, track_id);
				self.update_playlists();

				Task::none()
			}
			Message::ShowTrackList => {
				self.current_view = ViewType::Tracks;

				Task::none()
			}
			Message::DeleteAllTracks => {
				// DEBUG
				// for (kv) in self.db.to_owned().iter_tracks() {
				// 	let (id, t) = kv.unwrap();
				// 	self.db.delete_track(id);
				// }
				Task::none()
			}
			Message::ImportLocalTracks => {
				let tracks = self.get_local_tracks().unwrap();
				let mut track_ids = Vec::new();
				for t in tracks.iter() {
					let tid = self.db.to_owned().add_track(t);
					track_ids.push(tid.unwrap());
				}
				self.update_tracks();

				match &self.current_view {
					ViewType::Tracks => {}
					ViewType::Favorites => todo!(),
					ViewType::Playlist(playlist_id) => {
						for tid in track_ids.iter() {
							let _ = self.db.to_owned().add_track_to_playlist(*playlist_id, *tid);
						}
						self.update_playlists();
					}
					_ => {}
				}

				Task::none()
			}
			Message::AddTrackToTracks(track) => {
				let _ = self.db.to_owned().add_track(&track);

				Task::none()
			}
		}
	}

	fn request_track_audio_source(&self, track_id: &TrackId) -> Task<Message> {
		let track = self.db.get_track(*track_id).expect("Could not get track");
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
		.and_then(|res| Task::done(Message::AddTrackToControllerQueue(res)))
	}

	fn view(&self) -> Element<Message> {
		let width = 120.0;

		// playlist list
		let playlists = container(column(self.playlists.iter().filter_map(|id| {
			let p = self.db.get_playlist(*id).ok()?;
			Some(
				row![
					button(text(p.name.to_string())).on_press(Message::ShowPlaylist(*id)),
					button("DELETE")
						.width(width)
						.on_press(Message::DeletePlaylist(*id))
				]
				.into(),
			)
		})));

		// sidebar
		let current_view_text = match self.current_view {
			ViewType::Tracks => text("Tracks"),
			ViewType::Favorites => text("Favorites"),
			ViewType::Playlist(playlist_id) => {
				let p = self
					.db
					.get_playlist(playlist_id)
					.expect("Could not get playslist to show name");
				text(format!("Playlist : {}", p.name))
			}
			ViewType::Search => text("Search results"),
		};

		let sidebar = column![
			current_view_text,
			button("Import")
				.width(width)
				.on_press(Message::ImportLocalTracks),
			button("Delete ALL")
				.width(width)
				.on_press(Message::DeleteAllTracks),
			button("Tracks")
				.width(width)
				.on_press(Message::ShowTrackList),
			button("Tags").width(width).on_press(Message::ShowTags),
			text_input("Add playlist", &self.add_playlist_name)
				.width(width)
				.align_x(Center)
				.on_input(Message::AddPlaylistNameChange)
				.on_submit(Message::AddPlaylist),
			playlists,
		]
		.align_x(Center)
		.spacing(10);

		// tag filter bar
		let tag_filter_bar = text_input("tag filter", &self.tag_filter)
			.align_x(Center)
			.on_input(Message::QueryTagChange)
			.on_submit(Message::QueryTag);

		let track_list_view = match self.current_view {
			ViewType::Tracks => self.create_track_list_tracks(),
			ViewType::Favorites => todo!(),
			ViewType::Playlist(playlist_id) => self.create_track_list_playlist(playlist_id),
			ViewType::Search => self.create_track_list_search(),
		};

		// source_selector
		let source_selector = pick_list(
			[
				SearchSource::All,
				SearchSource::Soundcloud,
				SearchSource::Youtube,
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

		let media_bar: Row<_> = match &self.current_track_id {
			Some(tid) => {
				let t = self.db.get_track(*tid).expect("Could not get track");
				row![
					row![
						text("artwork"),
						column![
							text(format!("{}", t.title)),
							text(format!("{}", t.artists.join(", ")))
						]
					]
					.spacing(4.0),
					horizontal_space(),
					column![
						row![
							button("shuffle"),
							button("previous").on_press(Message::Previous),
							button("play_pause").on_press(Message::PlayPause),
							button("next").on_press(Message::Next),
							button("replay")
						]
						.spacing(4.0),
						text("track timer bar")
					],
					horizontal_space(),
					column![row![
						text("volume"),
						slider(0.0..=100.0, self.volume, Message::ChangeVolume)
					]
					.spacing(4.0)],
				]
				.align_y(Center)
			}
			None => row![text("No track.")],
		};

		let history_column = container(
			column(self.history.iter().rev().filter_map(|id| {
				let t = self.db.get_track(*id).ok()?;
				Some(
					row![
						text(t.artists.join(", ")),
						text(" - "),
						text(t.title.to_owned()),
					]
					.align_y(Center)
					.into(),
				)
			}))
			.spacing(20.0),
		)
		.center_x(Fill)
		.padding(20.0);

		let history_view = scrollable(history_column)
			.direction(scrollable::Direction::Vertical(
				scrollable::Scrollbar::new().width(1).scroller_width(10),
			))
			.width(Fill)
			.height(Fill);

		let queue_column = container(
			column(self.queue.iter().rev().filter_map(|id| {
				let t = self.db.get_track(*id).ok()?;
				Some(
					row![
						text(t.artists.join(", ")),
						text(" - "),
						text(t.title.to_owned()),
					]
					.align_y(Center)
					.into(),
				)
			}))
			.spacing(20.0),
		)
		.center_x(Fill)
		.padding(20.0);

		let queue_view = scrollable(queue_column)
			.direction(scrollable::Direction::Vertical(
				scrollable::Scrollbar::new().width(1).scroller_width(10),
			))
			.width(Fill)
			.height(Fill);

		container(column![
			row![
				sidebar,
				column![tag_filter_bar, track_list_view],
				column![history_view, queue_view]
			],
			row![source_selector, search_bar],
			media_bar,
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

	fn search_track_from_source(&self) -> Task<Message> {
		println!(
			"Search {:?} from source '{}'",
			self.search_query,
			self.search_source.to_string()
		);

		// TODO
		// match self.search_source {
		// 	SearchSource::All => todo!(),
		// 	SearchSource::Soundcloud => self.plugins,
		// 	SearchSource::Youtube => todo!(),
		// }

		let plugins = self.plugins.clone();
		todo!()
		// Task::perform(async move {
		// 	if let Some(result) = plugins
		// 		.iter()
		// 		.filter_map(|p| p.read().get_source_plugin())
		// 		.find_map(|p| p.handle_url(&url))
		// 	{
		// 		match result {
		// 			Ok(source) => Some(source),
		// 			Err(e) => {
		// 				warn!("error while handling track {url:?}: {e}");
		// 				Option::None
		// 			}
		// 		}
		// 	} else {
		// 		warn!("no plugin could handle the track: {url:?}");
		// 		Option::None
		// 	}
		// })
	}

	fn filter_tracks_by_tag_expression(&self) {
		println!(
			"Filter tracks with expression '{}'",
			self.tag_filter.to_string()
		)
	}
	fn create_track_list_tracks(&self) -> Scrollable<'_, Message> {
		let track_id_list = self.track_list.clone();

		let track_list_column = container(
			column(track_id_list.iter().filter_map(|id| {
				let t = self.db.get_track(*id).ok()?;
				Some(
					row![
						button("PLAY").on_press(Message::RequestPlayTrack(*id)),
						text(" "),
						text(t.artists.join(", ")),
						text(" - "),
						text(t.title.to_owned()),
						horizontal_space(),
						// TODO Error below, try to find why
						// pick_list(
						// 	self.playlists
						// 		.iter()
						// 		.map(|pid| (self.db.get_playlist(*pid).unwrap().name))
						// 		.collect::<Vec<String>>(),
						// 	Option::None::<String>,
						// 	move |pname| { Message::AddTrackToPlaylist(*id), pname) }
						// ),
						button("DELETE").on_press(Message::DeleteTrack(*id)),
					]
					.align_y(Center)
					.into(),
				)
			}))
			.spacing(20.0),
		)
		.center_x(Fill)
		.padding(20.0);

		let track_list_view = scrollable(track_list_column)
			.direction(scrollable::Direction::Vertical(
				scrollable::Scrollbar::new().width(1).scroller_width(10),
			))
			.width(Fill)
			.height(Fill);

		track_list_view
	}

	fn create_track_list_favorites(&self) -> Vec<TrackId> {
		todo!()
	}

	fn create_track_list_playlist(&self, playlist_id: PlaylistId) -> Scrollable<'_, Message> {
		let track_id_list = self
			.db
			.get_playlist(playlist_id)
			.expect("Could not get playlist")
			.track_ids;

		let track_list_column = container(
			column(track_id_list.iter().filter_map(|id| {
				let t = self.db.get_track(*id).ok()?;
				Some(
					row![
						button("PLAY").on_press(Message::RequestPlayTrack(*id)),
						text(" "),
						text(t.artists.join(", ")),
						text(" - "),
						text(t.title.to_owned()),
						horizontal_space(),
						// TODO Error below, try to find why
						// pick_list(
						// 	self.playlists
						// 		.iter()
						// 		.map(|pid| (self.db.get_playlist(*pid).unwrap().name))
						// 		.collect::<Vec<String>>(),
						// 	Option::None::<String>,
						// 	move |pname| { Message::AddTrackToPlaylist(*id), pname) }
						// ),
						button("DELETE")
							.on_press(Message::RemoveTrackFromPlaylist(playlist_id, *id)),
					]
					.align_y(Center)
					.into(),
				)
			}))
			.spacing(20.0),
		)
		.center_x(Fill)
		.padding(20.0);

		let track_list_view = scrollable(track_list_column)
			.direction(scrollable::Direction::Vertical(
				scrollable::Scrollbar::new().width(1).scroller_width(10),
			))
			.width(Fill)
			.height(Fill);

		track_list_view
	}

	fn create_track_list_search(&self) -> Scrollable<'_, Message> {
		let track_list: Vec<Track> = Vec::new();

		let track_list_column = container(
			column(track_list.iter().filter_map(|track| {
				Some(
					row![
						text("Title"),
						horizontal_space(),
						button("Add").on_press(Message::AddTrackToTracks(track.clone())),
					]
					.align_y(Center)
					.into(),
				)
			}))
			.spacing(20.0),
		)
		.center_x(Fill)
		.padding(20.0);

		let track_list_view = scrollable(track_list_column)
			.direction(scrollable::Direction::Vertical(
				scrollable::Scrollbar::new().width(1).scroller_width(10),
			))
			.width(Fill)
			.height(Fill);

		track_list_view
	}
}
