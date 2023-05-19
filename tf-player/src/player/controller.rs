use std::{
	sync::{
		atomic::{self, AtomicUsize},
		Arc,
	},
	time::Duration,
};

use anyhow::{anyhow, Result};
use parking_lot::RwLock;

use super::Command;
use crate::TrackSource;

#[derive(Clone)]
pub struct Controller {
	sender: crossbeam_channel::Sender<Command>,
	state: Arc<RwLock<super::State>>,
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
		sender: crossbeam_channel::Sender<Command>,
		nb_queued: Arc<AtomicUsize>,
	) -> Result<Self> {
		Ok(Self {
			state,
			sender,
			nb_queued,
		})
	}

	pub fn clear(&self) {
		self.sender.send(Command::Clear).unwrap();
	}

	pub fn queue_track(&self, source: TrackSource) -> Result<()> {
		if let Err(e) = self.sender.send(Command::QueueTrack(source)) {
			panic!("{:?}", e);
		}
		Ok(())
	}

	pub fn play(&self) -> Result<()> {
		self.sender
			.send(Command::Play)
			.map_err(|_| anyhow!("failed to play"))?;
		Ok(())
	}

	pub fn pause(&self) -> Result<()> {
		self.sender
			.send(Command::Pause)
			.map_err(|_| anyhow!("failed to pause"))?;
		Ok(())
	}

	pub fn play_pause(&self) -> Result<()> {
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

	pub fn seek(&self, position: Duration) -> Result<()> {
		self.sender.send(Command::Seek(position)).unwrap();
		Ok(())
	}

	pub fn skip(&self) -> Result<()> {
		self.sender.send(Command::Skip).unwrap();
		Ok(())
	}

	pub fn set_volume(&self, volume: f32) -> Result<()> {
		self.sender.send(Command::SetVolume(volume)).unwrap();
		Ok(())
	}

	pub fn state(&self) -> &RwLock<super::State> {
		&self.state
	}

	pub fn nb_queued(&self) -> usize {
		self.nb_queued.load(atomic::Ordering::Relaxed)
	}
}
