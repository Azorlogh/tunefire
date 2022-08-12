use std::{
	io::{Read, Seek, SeekFrom},
	sync::{atomic::AtomicBool, Arc, Mutex},
	time::Duration,
};

use anyhow::Result;
use symphonia::core::io::MediaSource;

use super::fetcher::Fetcher;

pub struct HttpProgressive {
	url: String,
	length: usize,
	position: usize,
	consumer: Mutex<rtrb::Consumer<u8>>,
	buffering: Arc<AtomicBool>,
}

impl HttpProgressive {
	pub fn new(url: &str, buffering: Arc<AtomicBool>) -> Result<Self> {
		let response = ureq::get(url).call()?;

		let length = response
			.header("Content-Length")
			.and_then(|r| r.parse::<usize>().ok())
			.unwrap_or(0);

		let consumer = Mutex::new(Fetcher::spawn(
			response.into_reader(),
			50000,
			buffering.clone(),
		)?);

		while consumer.lock().unwrap().slots() != 50000 {
			std::thread::sleep(Duration::from_millis(100));
		}

		Ok(Self {
			url: url.to_owned(),
			length,
			position: 0,
			consumer,
			buffering,
		})
	}
}

impl Read for HttpProgressive {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let len = self.consumer.lock().unwrap().read(buf)?;
		self.position += len;
		if len != buf.len() {
			self.buffering
				.store(true, std::sync::atomic::Ordering::Relaxed);
		}
		println!("{}", self.position);
		Ok(len)
	}
}

impl Seek for HttpProgressive {
	fn seek(&mut self, seek_from: SeekFrom) -> std::io::Result<u64> {
		// recreate a new reader for the new position
		let idx = match seek_from {
			SeekFrom::Start(idx) => idx as i64,
			SeekFrom::Current(off) => self.position as i64 + off,
			SeekFrom::End(off) => self.length as i64 - 1 + off,
		}
		.max(0) as u64;

		self.position = idx as usize;

		let response = ureq::get(&self.url)
			.set("Range", &format!("bytes={}-", idx))
			.call()
			.unwrap();

		let consumer = Mutex::new(
			Fetcher::spawn(response.into_reader(), 50000, self.buffering.clone()).unwrap(),
		);

		while consumer.lock().unwrap().slots() != 50000 {
			std::thread::sleep(Duration::from_millis(100));
		}

		self.consumer = consumer;

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
