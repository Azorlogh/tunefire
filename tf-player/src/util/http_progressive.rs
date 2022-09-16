use std::io::{Read, Seek, SeekFrom};

use anyhow::Result;
use symphonia::core::io::MediaSource;
use tracing::warn;

pub struct HttpProgressive {
	url: String,
	reader: Box<dyn Read + Sync + Send>,
	length: usize,
	position: usize,
}

impl HttpProgressive {
	pub fn new(url: &str) -> Result<Self> {
		let response = ureq::get(url).call()?;

		let length = response
			.header("Content-Length")
			.and_then(|r| r.parse::<usize>().ok())
			.unwrap_or(0);

		let position = 0;

		Ok(Self {
			url: url.to_owned(),
			reader: response.into_reader(),
			length,
			position,
		})
	}
}

impl Read for HttpProgressive {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let mut i = 0;
		let nb_read = loop {
			match self.reader.read(buf) {
				Err(e) => {
					warn!("failed to read http: {e}");
					if i == 5 {
						return Err(e);
					}
					warn!("retrying...{i}");
					let response = ureq::get(&self.url)
						.set("Range", &format!("bytes={}-", self.position))
						.call()
						.unwrap();
					self.reader = response.into_reader();
				}
				Ok(nb_read) => break nb_read,
			}
			i += 1;
		};
		self.position += nb_read;
		Ok(nb_read)
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

		warn!("seeking to {idx}");

		self.position = idx as usize;

		let response = ureq::get(&self.url)
			.set("Range", &format!("bytes={}-", idx))
			.call()
			.unwrap();

		self.reader = response.into_reader();

		Ok(idx)
	}
}

impl MediaSource for HttpProgressive {
	fn is_seekable(&self) -> bool {
		true // TODO: fix this
	}

	fn byte_len(&self) -> Option<u64> {
		Some(self.length as u64)
	}
}
