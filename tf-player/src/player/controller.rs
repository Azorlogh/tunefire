use std::{
	sync::{
		atomic::{self, AtomicUsize},
		Arc,
	},
	time::Duration,
};

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use url::Url;

use super::Event;
use crate::{LocalPlugin, SongSource, SoundcloudPlugin, SourcePlugin};

pub struct Controller {
	sender: crossbeam_channel::Sender<Event>,
	state: Arc<RwLock<super::State>>,
	plugins: Vec<Box<dyn SourcePlugin>>,
	_stream: cpal::Stream,
	nb_queued: Arc<AtomicUsize>,
}

impl std::fmt::Debug for Controller {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Player")
	}
}

impl Controller {
	pub fn new(
		state: Arc<RwLock<super::State>>,
		sender: crossbeam_channel::Sender<Event>,
		stream: cpal::Stream,
		nb_queued: Arc<AtomicUsize>,
	) -> Result<Self> {
		Ok(Self {
			state,
			sender,
			plugins: vec![Box::new(LocalPlugin), Box::new(SoundcloudPlugin::new()?)],
			_stream: stream,
			nb_queued,
		})
	}

	fn create_source(&self, url: &Url) -> Result<SongSource> {
		for plugin in &self.plugins {
			if let Some(source) = plugin.handle_url(&url) {
				let source = source.map_err(|err| {
					anyhow!(
						"plugin {} failed to handle this song {}",
						plugin.name(),
						err
					)
				})?;
				return Ok(source);
			}
		}
		Err(anyhow!("no plugin could find this song"))
	}

	pub fn queue_song(&mut self, url: Url) -> Result<()> {
		let source = self.create_source(&url)?;
		if let Err(e) = self.sender.send(Event::QueueSong(source)) {
			panic!("{:?}", e);
		}
		Ok(())
	}

	pub fn play(&mut self) -> Result<()> {
		self.sender
			.send(Event::Play)
			.map_err(|_| anyhow!("failed to play"))?;
		Ok(())
	}

	pub fn pause(&mut self) -> Result<()> {
		self.sender
			.send(Event::Pause)
			.map_err(|_| anyhow!("failed to pause"))?;
		Ok(())
	}

	pub fn play_pause(&mut self) -> Result<()> {
		if self
			.state
			.read()
			.get_playing()
			.ok_or(anyhow!("wrong state"))?
			.paused
		{
			self.play()
		} else {
			self.pause()
		}
	}

	pub fn seek(&mut self, position: Duration) -> Result<()> {
		self.sender.send(Event::Seek(position)).unwrap();
		Ok(())
	}

	pub fn state(&self) -> &RwLock<super::State> {
		&self.state
	}

	pub fn nb_queued(&self) -> usize {
		self.nb_queued.load(atomic::Ordering::Relaxed)
	}
}
