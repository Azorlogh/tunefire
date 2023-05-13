use std::{sync::mpsc::Receiver, time::Duration};

use anyhow::{anyhow, Result};
#[cfg(target_os = "windows")]
use druid::HasRawWindowHandle;
use souvlaki::{MediaControlEvent, MediaMetadata, MediaPlayback, MediaPosition};

pub struct MediaControls {
	pub controls: souvlaki::MediaControls,
	pub events: Receiver<MediaControlEvent>,
}

impl MediaControls {
	pub fn new(#[allow(unused)] window_handle: &druid::WindowHandle) -> Result<Self> {
		#[cfg(not(target_os = "windows"))]
		let hwnd = None;
		#[cfg(target_os = "windows")]
		let hwnd = match window_handle.raw_window_handle() {
			druid::RawWindowHandle::Win32(h) => Some(h.hwnd),
			_ => panic!("no window handle"),
		};

		let config = souvlaki::PlatformConfig {
			dbus_name: "tunefire",
			display_name: "Tunefire",
			hwnd,
		};

		let mut controls = souvlaki::MediaControls::new(config)
			.map_err(|e| anyhow!("failed create platform media controls: {:?}", e))?;

		let (to_handler, from_media_events) = std::sync::mpsc::sync_channel(32);

		controls
			.attach(move |e| {
				to_handler.send(e).ok();
			})
			.map_err(|e| anyhow!("failed attach media control handler: {:?}", e))?;

		Ok(Self {
			controls,
			events: from_media_events,
		})
	}

	pub fn set_metadata(&mut self, artist: &str, title: &str) -> Result<()> {
		self.controls
			.set_metadata(MediaMetadata {
				artist: Some(artist),
				title: Some(title),
				..Default::default()
			})
			.map_err(|e| anyhow!("failed to set metadata {:?}", e))
	}

	pub fn set_is_playing(&mut self, playing: bool) -> Result<()> {
		if playing {
			self.controls.set_playback(MediaPlayback::Playing {
				progress: Some(MediaPosition(Duration::from_secs(0))),
			})
		} else {
			self.controls.set_playback(MediaPlayback::Stopped)
		}
		.map_err(|e| anyhow!("failed to set playback {:?}", e))
	}
}
