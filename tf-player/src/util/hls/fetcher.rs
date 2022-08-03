use std::{io::Read, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use parking_lot::Mutex;
use tracing::{debug, instrument, trace};

use super::{SegmentCache, SegmentInfos};

#[derive(Debug)]
pub struct Fetcher {
	duration: Duration,
	segment_infos: SegmentInfos,
	segment_cache: Arc<Mutex<SegmentCache>>,
}

impl Fetcher {
	pub fn spawn(
		duration: Duration,
		segment_infos: SegmentInfos,
		segment_cache: Arc<Mutex<SegmentCache>>,
	) -> Result<()> {
		// grab the first segment to help symphonia probe the format
		let first_empty = segment_cache.lock().segments[0].is_none();
		if first_empty {
			segment_cache.lock().segments[0] =
				Some(fetch_segment_retry(&segment_infos.0[0].url, 5)?);
		}

		let fetcher = Fetcher {
			duration,
			segment_infos,
			segment_cache,
		};
		std::thread::spawn(move || {
			if let Err(e) = fetcher.run() {
				println!("{e}");
			}
		});
		Ok(())
	}

	#[instrument(skip_all)]
	fn run(self) -> Result<()> {
		loop {
			// We look where the source is located and make sure the following 10 seconds are loaded in cache
			let source_position = self.segment_cache.lock().source_position;
			trace!("fetching iteration at position {:?}", source_position);
			let curr_time = self
				.segment_infos
				.time_at_position(source_position.0, source_position.1);
			let curr_segment = self
				.segment_infos
				.segment_at(curr_time)
				.map(|p| p.0)
				.unwrap();
			let target_time = (curr_time + Duration::from_secs(10)).min(self.duration);
			let target_segment = self
				.segment_infos
				.segment_at(target_time)
				.map(|p| p.0)
				.unwrap_or(self.segment_infos.0.len());
			for idx in curr_segment..=target_segment {
				if self.segment_cache.lock().segments.get(idx) == Some(&None) {
					trace!("fetching next segment: {}", idx);
					let info = &self.segment_infos.0[idx];
					self.segment_cache.lock().segments[idx] =
						Some(fetch_segment_retry(&info.url, 5)?);
					trace!("next segment received!");
					if idx == self.segment_cache.lock().source_position.0 {
						debug!("STOPPED BUFFERING");
						self.segment_cache.lock().buffering = false;
					}
				}
			}
			std::thread::sleep(Duration::from_secs(1));
		}
	}
}

fn fetch_segment_retry(url: &str, nb_tries: usize) -> Result<Vec<u8>> {
	for _ in 0..nb_tries {
		if let Ok(s) = fetch_segment(url) {
			return Ok(s);
		}
		std::thread::sleep(Duration::from_secs(1));
	}
	fetch_segment(url).context("Couldn't fetch segment")
}

fn fetch_segment(url: &str) -> Result<Vec<u8>> {
	Ok(ureq::get(url)
		.call()?
		.into_reader()
		.bytes()
		.collect::<Result<Vec<u8>, std::io::Error>>()?)
}
