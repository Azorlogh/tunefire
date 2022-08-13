use std::{
	io::{Read, Seek, SeekFrom},
	sync::{
		atomic::{self, AtomicBool, AtomicUsize},
		Arc, Mutex,
	},
	time::Duration,
};

use anyhow::Result;
use crossbeam_channel::Sender;
use symphonia::core::io::MediaSource;
use tracing::{debug, error};

use self::{
	cache::Cache,
	fetcher::{Event, Fetcher},
};

mod cache;
mod fetcher;

const HEADROOM: usize = 128000;

pub struct HttpProgressive {
	url: String,
	length: usize,
	position: Arc<AtomicUsize>,
	cache: Arc<Mutex<Cache>>,
	buffering: Arc<AtomicBool>,
	to_fetcher: Sender<Event>,
}

impl HttpProgressive {
	pub fn new(url: &str, buffering: Arc<AtomicBool>) -> Result<Self> {
		let response = ureq::get(url).call()?;

		let length = response
			.header("Content-Length")
			.and_then(|r| r.parse::<usize>().ok())
			.unwrap_or(0);

		println!("{} bytes", length);
		dbg!(response.status());

		let cache = Arc::new(Mutex::new(Cache::new(length)));

		let position = Arc::new(AtomicUsize::new(0));

		let to_fetcher = Fetcher::spawn(
			response.into_reader(),
			cache.clone(),
			buffering.clone(),
			position.clone(),
		)?;

		println!("all seems good!");

		Ok(Self {
			url: url.to_owned(),
			length,
			position,
			cache,
			to_fetcher,
			buffering,
		})
	}
}

impl Read for HttpProgressive {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let position = self.position.load(atomic::Ordering::Relaxed);

		let mut cache = self.cache.lock().unwrap();
		let available = cache.available_from(position);
		println!(
			"available: {}, while we want {}",
			available.len(),
			buf.len()
		);
		if available.len() > buf.len() {
			buf.copy_from_slice(&available[..buf.len()]);
			println!("read {} bytes from cache", buf.len());
			self.position
				.store(position + buf.len(), atomic::Ordering::Relaxed);
			Ok(buf.len())
		} else {
			self.buffering.store(true, atomic::Ordering::Relaxed);
			Err(std::io::Error::new(
				std::io::ErrorKind::WouldBlock,
				"Buffering...",
			))
		}
	}
}

impl Seek for HttpProgressive {
	fn seek(&mut self, seek_from: SeekFrom) -> std::io::Result<u64> {
		let position = self.position.load(atomic::Ordering::Relaxed);

		// recreate a new reader for the new position
		let idx = match seek_from {
			SeekFrom::Start(idx) => idx as i64,
			SeekFrom::Current(off) => position as i64 + off,
			SeekFrom::End(off) => self.length as i64 - 1 + off,
		}
		.max(0) as u64;

		self.position.store(idx as usize, atomic::Ordering::Relaxed);
		self.cache.lock().unwrap().seek(idx as usize);

		let response = ureq::get(&self.url)
			.set("Range", &format!("bytes={}-", idx))
			.call()
			.unwrap();

		self.to_fetcher
			.send(Event::SetReader(response.into_reader()))
			.unwrap();

		while self
			.cache
			.lock()
			.unwrap()
			.available_from(idx as usize)
			.len() < 50000
		{
			std::thread::sleep(Duration::from_millis(100));
		}

		Ok(idx)
	}
}

impl MediaSource for HttpProgressive {
	fn is_seekable(&self) -> bool {
		false // TODO: fix this
	}

	fn byte_len(&self) -> Option<u64> {
		Some(self.length as u64)
	}
}
