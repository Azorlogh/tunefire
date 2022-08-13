use std::{
	io::Read,
	sync::{
		atomic::{self, AtomicBool, AtomicUsize},
		Arc, Mutex,
	},
	time::Duration,
};

pub enum Event {
	SetReader(Box<dyn Read + Send>),
	Stop,
}

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use tracing::error;

use super::{cache::Cache, HEADROOM};

pub struct Fetcher {
	reader: Box<dyn Read + Send>,
	cache: Arc<Mutex<Cache>>,
	buffering: Arc<AtomicBool>,
	position: Arc<AtomicUsize>,
	receiver: Receiver<Event>,
}

impl Fetcher {
	pub fn spawn(
		reader: Box<dyn Read + Send>,
		cache: Arc<Mutex<Cache>>,
		buffering: Arc<AtomicBool>,
		reader_position: Arc<AtomicUsize>,
	) -> Result<Sender<Event>> {
		let (sender, receiver) = crossbeam_channel::unbounded();

		let fetcher = Fetcher {
			reader,
			cache,
			buffering,
			receiver,
			position: reader_position,
		};

		std::thread::spawn(|| {
			fetcher.run().unwrap();
		});

		Ok(sender)
	}

	pub fn run(mut self) -> Result<()> {
		loop {
			while let Ok(evt) = self.receiver.try_recv() {
				match evt {
					Event::SetReader(reader) => {
						self.reader = reader;
					}
					Event::Stop => {
						return Ok(());
					}
				}
			}

			let mut cache = self.cache.lock().unwrap();
			if cache
				.available_from(self.position.load(atomic::Ordering::Relaxed))
				.len() < HEADROOM
			{
				cache.fill(&mut self.reader).unwrap();
				println!(
					"{:?}",
					cache
						.available_from(self.position.load(atomic::Ordering::Relaxed))
						.len()
				);
			} else {
				self.buffering.store(false, atomic::Ordering::Relaxed);
				std::thread::sleep(Duration::from_secs(1));
			}
		}
	}
}
