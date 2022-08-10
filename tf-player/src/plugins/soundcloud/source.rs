use std::{sync::Arc, time::Duration};

use anyhow::Result;
use hls_m3u8::MediaPlaylist;
use parking_lot::Mutex;
use symphonia::core::{io::MediaSourceStream, probe::Hint};

use crate::{
	util::{
		self,
		hls::{self, SegmentCache, SegmentInfos},
	},
	Source,
};

pub struct SoundcloudSource {
	segment_infos: SegmentInfos,
	cache: Arc<Mutex<SegmentCache>>,
	pub source: util::symphonia::Source,
	seeking: Option<Duration>,
}

impl SoundcloudSource {
	pub fn new(media_playlist: &MediaPlaylist) -> Result<Self> {
		let duration = media_playlist.duration();
		let segment_infos = SegmentInfos::from_hls(&media_playlist);
		let cache = Arc::new(Mutex::new(SegmentCache::new(segment_infos.0.len())));

		hls::Fetcher::spawn(duration, segment_infos.clone(), cache.clone())?;

		let hls_source = hls::MediaSource::new(
			duration,
			segment_infos.clone(),
			cache.clone(),
			Duration::ZERO,
		);

		let mss = MediaSourceStream::new(Box::new(hls_source), Default::default());
		let symphonia_source = util::symphonia::Source::from_mss(mss, Hint::new())?;

		Ok(Self {
			segment_infos,
			cache,
			source: symphonia_source,
			seeking: None,
		})
	}
}

impl Source for SoundcloudSource {
	fn seek(&mut self, pos: Duration) -> Result<(), crate::SourceError> {
		match self.source.seek(pos) {
			Err(crate::SourceError::Buffering) => {
				let (curr_segment_idx, mut curr_offset) =
					self.segment_infos.segment_at(pos).unwrap();
				curr_offset = curr_offset.saturating_sub(5000); // safety to avoid not having enough data for symphonia
				self.cache.lock().source_position = (curr_segment_idx, curr_offset);
				self.seeking = Some(pos);
				return Err(crate::SourceError::Buffering);
			}
			_ => {}
		}
		Ok(())
	}

	fn next(&mut self, buf: &mut [[f32; 2]]) -> Result<(), crate::SourceError> {
		if let Some(pos) = self.seeking {
			if !self.cache.lock().buffering {
				self.seeking = None;
				let hls_source = hls::MediaSource::new(
					self.source.duration,
					self.segment_infos.clone(),
					self.cache.clone(),
					pos,
				);
				let mss = MediaSourceStream::new(Box::new(hls_source), Default::default());
				self.source = util::symphonia::Source::from_mss(mss, Hint::new()).unwrap();
			} else {
				return Err(crate::SourceError::Buffering);
			}
		}
		self.source.next(buf)
	}
}
