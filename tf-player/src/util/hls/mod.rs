use std::time::Duration;

use hls_m3u8::MediaPlaylist;

mod fetcher;
mod source;

pub use fetcher::Fetcher;
pub use source::MediaSource;

pub const BYTERATE: f64 = 128_000.0 / 8.0;

// We need a thread that fetches the HLS segments continuously as we stream the audio
// and inserts them into a cache protected by a mutex.
// And the SoundcloudMediaSource needs to pull the segments from the cache.
// The fetcher will try to keep the cache filled for the next 10 seconds

#[derive(Debug, Clone)]
pub struct SegmentInfo {
	url: String,
	duration: Duration,
}

#[derive(Debug)]
pub struct SegmentCache {
	pub source_position: (usize, usize), // (segment idx, byte idx)
	pub segments: Vec<Option<Vec<u8>>>,
	pub buffering: bool,
}

impl SegmentCache {
	pub fn new(nb_segments: usize) -> Self {
		Self {
			source_position: (0, 0),
			segments: vec![None; nb_segments],
			buffering: false,
		}
	}
}

#[derive(Debug, Clone)]
pub struct SegmentInfos(pub Vec<SegmentInfo>);

impl SegmentInfos {
	pub fn from_hls(hls: &MediaPlaylist) -> Self {
		Self(
			hls.segments
				.values()
				.map(|segment| SegmentInfo {
					url: segment.uri().to_string(),
					duration: segment.duration.duration(),
				})
				.collect(),
		)
	}

	pub fn segment_at(&self, time: Duration) -> Option<(usize, usize)> {
		let mut t = Duration::ZERO;
		for (idx, segment) in self.0.iter().enumerate() {
			t += segment.duration;
			if t > time {
				let byte_offset =
					((time - (t - segment.duration)).as_secs_f64() * BYTERATE) as usize;
				return Some((idx, byte_offset));
			}
		}
		return None;
	}

	pub fn time_at_position(&self, segment_idx: usize, byte_idx: usize) -> Duration {
		let mut t = Duration::ZERO;
		for (idx, segment) in self.0.iter().enumerate() {
			if segment_idx == idx {
				t += Duration::from_secs_f64(byte_idx as f64 / BYTERATE);
				break;
			}
			t += segment.duration;
		}
		return t;
	}
}
