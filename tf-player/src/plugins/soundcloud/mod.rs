use std::convert::TryFrom;

use anyhow::{anyhow, Result};
use tracing::debug;
use url::Url;

use crate::{SourcePlugin, TrackInfo, TrackSource};

mod api;
mod source;

pub struct SoundcloudPlugin {
	client_id: String,
}

impl SoundcloudPlugin {
	pub fn new() -> Result<Self> {
		// grap soundcloud home page
		let body = ureq::get("https://soundcloud.com").call()?.into_string()?;

		// grab location of a specific script
		let re = regex::Regex::new(r#"<script crossorigin src="([^\n]*)">"#).unwrap();
		let magic_script = &re
			.captures_iter(&body)
			.last()
			.ok_or(anyhow!("could not find url to the magic script"))?[1];

		// grab that script
		let body = ureq::get(magic_script).call()?.into_string()?;
		let re = regex::Regex::new(r#"client_id:"([^"]*)""#).unwrap();

		let client_id = re
			.captures(&body)
			.ok_or(anyhow!("missing client id in script"))?[1]
			.to_owned();

		debug!("soundcloud client id: {:?}", client_id);

		Ok(Self { client_id })
	}

	pub fn handle(&self, url: &Url) -> Result<TrackSource> {
		let resolve: api::ResolveResponse = serde_json::from_str(
			&ureq::get(&format!(
				"https://api-v2.soundcloud.com/resolve?client_id={}&url={}",
				self.client_id, url
			))
			.call()?
			.into_string()?,
		)?;

		let media_url = format!(
			"{}?client_id={}&track_authorization={}",
			resolve.media.transcodings[0].url, self.client_id, resolve.track_authorization
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

		let source = source::SoundcloudSource::new(&hls)?;

		Ok(TrackSource {
			info: TrackInfo {
				duration: source.source.duration,
			},
			sample_rate: source.source.sample_rate,
			signal: Box::new(source),
		})
	}
}

impl SourcePlugin for SoundcloudPlugin {
	fn name(&self) -> &'static str {
		"Soundcloud"
	}

	fn handle_url(&self, url: &Url) -> Option<Result<TrackSource>> {
		(url.scheme() == "https" && url.host_str() == Some("soundcloud.com"))
			.then(|| self.handle(url))
	}
}
