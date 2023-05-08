use std::time::Duration;

use anyhow::{anyhow, Result};
use symphonia::core::{io::MediaSourceStream, probe::Hint};
use tf_plugin::player::{
	util::{self, http_progressive::HttpProgressive},
	Source, SourceError, SourcePlugin, TrackInfo, TrackSource,
};
use tokio::runtime::Runtime;
use tracing::debug;
use url::Url;

pub struct YoutubeSourcePlugin {
	pub client: ytextract::Client,
}

impl YoutubeSourcePlugin {
	pub fn handle(&self, url: &Url) -> Result<TrackSource> {
		let video_id: ytextract::video::Id = url
			.query_pairs()
			.find(|pair| pair.0 == "v")
			.ok_or(anyhow!("video id missing from the url"))?
			.1
			.parse()?;

		let rt = Runtime::new().unwrap();
		let (video, stream) = rt.block_on(async {
			let video = self.client.video(video_id).await?;
			let streams = video
				.streams()
				.await
				.unwrap()
				.filter(|s| s.is_audio())
				.collect::<Vec<_>>();
			Result::<_, anyhow::Error>::Ok((video, streams[1].clone()))
		})?;

		let duration = video.duration();

		let source = YoutubeSource::new(&stream.url())?;

		Ok(TrackSource {
			info: TrackInfo { duration },
			sample_rate: 44100.0,
			signal: Box::new(source),
		})
	}
}

impl SourcePlugin for YoutubeSourcePlugin {
	fn name(&self) -> &'static str {
		"Youtube"
	}

	fn handle_url(&self, url: &Url) -> Option<Result<TrackSource>> {
		(url.scheme() == "https" && url.host_str() == Some("www.youtube.com"))
			.then(|| self.handle(url))
	}
}

pub struct YoutubeSource {
	pub source: util::symphonia::Source,
}

impl YoutubeSource {
	pub fn new(url: &Url) -> Result<Self> {
		let media_source = HttpProgressive::new(url.as_str())?;

		debug!("created media source");

		let mss = MediaSourceStream::new(Box::new(media_source), Default::default());
		let mut hint = Hint::new();
		hint.mime_type("audio/aac");
		let source = util::symphonia::Source::from_mss(mss, hint)?;

		debug!("created symphonia source");

		Ok(Self { source })
	}
}

impl Source for YoutubeSource {
	fn seek(&mut self, pos: Duration) -> Result<(), SourceError> {
		self.source.seek(pos)
	}

	fn next(&mut self, buf: &mut [[f32; 2]]) -> Result<(), SourceError> {
		self.source.next(buf)
	}
}
