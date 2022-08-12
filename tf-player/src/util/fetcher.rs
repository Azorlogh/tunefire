use std::{
	io::Read,
	sync::{atomic::AtomicBool, Arc},
	time::Duration,
};

use anyhow::Result;

pub struct Fetcher<R: Read> {
	reader: R,
	prod: rtrb::Producer<u8>,
	buffering: Arc<AtomicBool>,
}

impl<R: Read + Send + 'static> Fetcher<R> {
	pub fn spawn(
		reader: R,
		capacity: usize,
		buffering: Arc<AtomicBool>,
	) -> Result<rtrb::Consumer<u8>> {
		let (prod, cons) = rtrb::RingBuffer::new(capacity);

		let fetcher = Fetcher {
			reader,
			prod,
			buffering,
		};

		std::thread::spawn(|| {
			fetcher.run().unwrap();
		});

		Ok(cons)
	}

	pub fn run(mut self) -> Result<()> {
		while !self.prod.is_abandoned() {
			let slots = self.prod.slots();
			if slots == 0 {
				std::thread::sleep(Duration::from_millis(500));
				self.buffering
					.store(false, std::sync::atomic::Ordering::Relaxed);
				continue;
			}
			let mut chunk = self.prod.write_chunk(slots)?;
			let (first, _second) = chunk.as_mut_slices();
			let n_read = self.reader.read(first)?;
			// filling second buffer causes a strange error
			chunk.commit(n_read);
		}
		println!("fetcher was abandonned, stopping");
		Ok(())
	}
}
