use std::{sync::Arc, time::Duration};

use anyhow::Result;
use hls_m3u8::MediaPlaylist;
use parking_lot::Mutex;
use symphonia::core::{formats::FormatReader, io::MediaSourceStream};
use tf_plugin::player::{
	util::{
		self,
		hls::{self, SegmentCache, SegmentInfos},
	},
	Source, SourceError,
};

mod plugin;
pub use plugin::SoundcloudSourcePlugin;

pub struct SoundcloudSource {
	segment_infos: SegmentInfos,
	cache: Arc<Mutex<SegmentCache>>,
	pub source: util::symphonia::Source,
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
		let format = symphonia::default::formats::MpaReader::try_new(mss, &Default::default())?;
		let symphonia_source = util::symphonia::Source::from_format_reader(Box::new(format))?;

		Ok(Self {
			segment_infos,
			cache,
			source: symphonia_source,
		})
	}
}

impl Source for SoundcloudSource {
	fn seek(&mut self, pos: Duration) -> Result<(), SourceError> {
		// Once symphonia's Mp3Reader supports SeekMode::Coarse,
		// we can just replace this by: self.source.seek(pos)
		// Until then, we recreate the source at the new location
		let hls_source = hls::MediaSource::new(
			self.source.duration,
			self.segment_infos.clone(),
			self.cache.clone(),
			pos,
		);
		let mss = MediaSourceStream::new(Box::new(hls_source), Default::default());
		let format =
			symphonia::default::formats::MpaReader::try_new(mss, &Default::default()).unwrap();
		self.source = util::symphonia::Source::from_format_reader(Box::new(format)).unwrap();
		Ok(())
	}

	fn next(&mut self, buf: &mut [[f32; 2]]) -> Result<(), SourceError> {
		self.source.next(buf)
	}
}
