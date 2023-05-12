use std::convert::TryFrom;

use anyhow::{anyhow, Result};
use tf_plugin::player::{SourcePlugin, TrackInfo, TrackSource};
use url::Url;

use crate::api;

pub struct SoundcloudSourcePlugin {
	pub client_id: String,
}

impl SoundcloudSourcePlugin {
	pub fn handle(&self, url: &Url) -> Result<TrackSource> {
		let api::ResolveResponse::Track(resolved) = serde_json::from_str(
			&ureq::get(&format!(
				"https://api-v2.soundcloud.com/resolve?client_id={}&url={}",
				self.client_id, url
			))
			.call()?
			.into_string()?,
		)? else {
			return Err(anyhow!("url is not a track"))
		};

		let media_url = format!(
			"{}?client_id={}&track_authorization={}",
			resolved.media.transcodings[0].url, self.client_id, resolved.track_authorization
		);

		let media_response: api::MediaResponse =
			serde_json::from_str(&ureq::get(&media_url).call().unwrap().into_string().unwrap())
				.unwrap();

		let hls_str = ureq::get(&media_response.url)
			.call()
			.unwrap()
			.into_string()
			.unwrap();

		let hls = hls_m3u8::MediaPlaylist::try_from(hls_str.as_str()).unwrap();

		let source = super::SoundcloudSource::new(&hls)?;

		Ok(TrackSource {
			info: TrackInfo {
				duration: source.source.duration,
			},
			sample_rate: source.source.sample_rate,
			signal: Box::new(source),
		})
	}
}

impl SourcePlugin for SoundcloudSourcePlugin {
	fn name(&self) -> &'static str {
		"Soundcloud"
	}

	fn handle_url(&self, url: &Url) -> Option<Result<TrackSource>> {
		(url.scheme() == "https" && url.host_str() == Some("soundcloud.com"))
			.then(|| self.handle(url))
	}
}
