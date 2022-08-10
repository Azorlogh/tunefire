use std::io::{Read, Seek, SeekFrom};

use anyhow::Result;
use symphonia::core::io::MediaSource;

pub struct HttpProgressive {
	url: String,
	length: usize,
	position: usize,
	reader: Box<dyn Read + Send + Sync>,
}

impl HttpProgressive {
	pub fn new(url: &str) -> Result<Self> {
		let response = ureq::get(url).call()?;

		let length = response
			.header("Content-Length")
			.and_then(|r| r.parse::<usize>().ok())
			.unwrap_or(0);

		Ok(Self {
			url: url.to_owned(),
			length,
			position: 0,
			reader: response.into_reader(),
		})
	}
}

impl Read for HttpProgressive {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let len = self.reader.read(buf)?;
		self.position += len;
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

		self.reader = ureq::get(&self.url)
			.set("Range", &format!("bytes={}-", idx))
			.call()
			.unwrap()
			.into_reader();

		Ok(idx)
	}
}

impl MediaSource for HttpProgressive {
	fn is_seekable(&self) -> bool {
		true
	}

	fn byte_len(&self) -> Option<u64> {
		Some(self.length as u64)
	}
}
