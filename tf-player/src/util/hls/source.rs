use std::{
	io::{Read, Seek, SeekFrom},
	sync::Arc,
	time::Duration,
};

use parking_lot::Mutex;
use tracing::{debug, instrument, trace};

use super::{SegmentCache, SegmentInfos};
use crate::util::hls::BYTERATE;

pub struct MediaSource {
	duration: Duration,
	segment_infos: SegmentInfos,
	segment_cache: Arc<Mutex<SegmentCache>>,
	curr_segment: Option<Vec<u8>>,
	curr_segment_idx: usize,
	curr_offset: usize,
}

impl MediaSource {
	pub fn new(
		duration: Duration,
		segment_infos: SegmentInfos,
		segment_cache: Arc<Mutex<SegmentCache>>,
		pos: Duration,
	) -> Self {
		let (curr_segment_idx, curr_offset) = segment_infos.segment_at(pos).unwrap();
		if segment_cache.lock().segments[curr_segment_idx].is_some() {
			segment_cache.lock().buffering = false;
		}
		segment_cache.lock().source_position = (curr_segment_idx, curr_offset);

		debug!(
			"Creating hls::MediaSource with {:?}/{:?}",
			curr_segment_idx, curr_offset
		);

		Self {
			duration,
			segment_infos,
			segment_cache,
			curr_segment: None,
			curr_segment_idx,
			curr_offset,
		}
	}

	#[instrument(skip_all)]
	fn load_segment(&mut self) -> std::io::Result<()> {
		trace!(
			"grabbing segment {} from the cache...",
			self.curr_segment_idx
		);

		let segment = loop {
			let cache = self.segment_cache.lock();
			match &cache.segments[self.curr_segment_idx] {
				Some(s) => break s.clone(),
				None => {
					debug!("failed to grab segment, retrying...");
				}
			}
			std::thread::sleep(Duration::from_millis(1000));
		};
		self.curr_segment = Some(segment);
		Ok(())
	}

	fn advance(&mut self) -> std::io::Result<()> {
		self.curr_segment_idx += 1;
		self.curr_segment = None;
		if self.curr_segment_idx < self.segment_infos.0.len() {
			self.load_segment()?;
			self.curr_offset = 0;
			Ok(())
		} else {
			Err(std::io::Error::new(
				std::io::ErrorKind::UnexpectedEof,
				"End of stream",
			))
		}
	}

	fn ended(&self) -> bool {
		self.curr_segment_idx >= self.segment_infos.0.len()
	}
}

impl Read for MediaSource {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		if self.curr_segment.is_none() {
			self.load_segment()?;
			// self.curr_offset = 0;
		}

		let mut nb_filled = 0;
		while nb_filled < buf.len() && !self.ended() {
			let segment = self.curr_segment.as_ref().unwrap();

			let nb_remaining_to_take = buf.len() - nb_filled;
			// debug!("{:?} - {:?}", segment.len(), self.curr_offset);
			let nb_available_in_segment = segment.len().saturating_sub(self.curr_offset);
			let nb_to_take = nb_remaining_to_take.min(nb_available_in_segment);

			let segment_slice = &segment[self.curr_offset..][..nb_to_take];
			buf[nb_filled..nb_filled + nb_to_take].copy_from_slice(segment_slice);
			self.curr_offset += nb_to_take;
			nb_filled += nb_to_take;

			if nb_filled < buf.len() {
				self.advance()?;
			}
		}

		self.segment_cache.lock().source_position = (self.curr_segment_idx, self.curr_offset);

		Ok(nb_filled)
	}
}

impl Seek for MediaSource {
	fn seek(&mut self, seek_from: std::io::SeekFrom) -> std::io::Result<u64> {
		let curr_pos = (self
			.segment_infos
			.time_at_position(self.curr_segment_idx, self.curr_offset)
			.as_secs_f64()
			* BYTERATE) as i64;

		let idx = match seek_from {
			SeekFrom::Start(idx) => idx as i64,
			SeekFrom::Current(off) => curr_pos + off,
			SeekFrom::End(off) => (self.duration.as_secs_f64() * BYTERATE) as i64 - off,
		}
		.max(0) as u64;

		let time = Duration::from_secs_f64(idx as f64 / BYTERATE);

		let segment_idx = self
			.segment_infos
			.segment_at(time)
			.map(|p| p.0)
			.unwrap_or(self.segment_infos.0.len() - 1);

		let segment_start = self.segment_infos.time_at_position(segment_idx, 0);

		let offset = ((time - segment_start).as_secs_f64() * BYTERATE) as u64;

		self.curr_segment_idx = segment_idx;
		self.curr_offset = offset as usize;
		self.curr_segment = None;
		self.segment_cache.lock().source_position = (self.curr_segment_idx, self.curr_offset);

		println!(
			"HLS SOURCE SEEKED TO {}/{}",
			self.curr_segment_idx, self.curr_offset
		);

		Ok(idx)
	}
}

impl symphonia::core::io::MediaSource for MediaSource {
	fn is_seekable(&self) -> bool {
		true
	}

	fn byte_len(&self) -> Option<u64> {
		Some((BYTERATE * self.duration.as_secs_f64()).ceil() as u64)
	}
}
