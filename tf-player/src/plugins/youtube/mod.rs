use std::sync::mpsc;

use anyhow::{anyhow, Result};
use symphonia::core::{io::MediaSourceStream, probe::Hint};
use tokio::runtime::Runtime;
use url::Url;

use crate::{
	util::{self, http_progressive::HttpProgressive},
	SongInfo, SongSource, SourcePlugin,
};

pub struct YoutubePlugin {
	client: ytextract::Client,
}

impl YoutubePlugin {
	pub fn new() -> Result<Self> {
		Ok(YoutubePlugin {
			client: ytextract::Client::new(),
		})
	}

	pub fn handle(&self, url: &Url) -> Result<SongSource> {
		let re = regex::Regex::new(r#"\?v=(.*)"#).unwrap();
		let video_id: ytextract::video::Id = re
			.captures_iter(&url.as_str())
			.last()
			.ok_or(anyhow!("could not find url to the magic script"))?[1]
			.parse()?;

		let rt = Runtime::new().unwrap();
		let (tx, rx) = mpsc::channel();
		rt.block_on(async {
			let video = self.client.video(video_id).await.unwrap();
			let streams = video
				.streams()
				.await
				.unwrap()
				.filter(|s| s.is_audio())
				.collect::<Vec<_>>();
			tx.send((video, streams[1].clone())).unwrap();
		});
		let (video, stream) = rx.recv().unwrap();

		let duration = video.duration();

		let media_source = HttpProgressive::new(stream.url().as_str())?;

		let mss = MediaSourceStream::new(Box::new(media_source), Default::default());
		let mut hint = Hint::new();
		hint.mime_type("audio/aac");
		let source = util::symphonia::Source::from_mss(mss, hint)?;

		Ok(SongSource {
			info: SongInfo { duration },
			sample_rate: 44100.0,
			signal: Box::new(source),
		})
	}
}

impl SourcePlugin for YoutubePlugin {
	fn name(&self) -> &'static str {
		"Youtube"
	}

	fn handle_url(&self, url: &Url) -> Option<Result<SongSource>> {
		(url.scheme() == "https" && url.host_str() == Some("www.youtube.com"))
			.then(|| self.handle(url))
	}
}
