use std::{iter::once, sync::Arc};

use anyhow::{anyhow, Result};
use druid::im;
use futures::StreamExt;
use tf_plugin::{ImportPlugin, ImportedItem, TrackInfo};
use tokio::runtime::Runtime;
use url::Url;

pub struct YoutubeImportPlugin {
	pub client: ytextract::Client,
}

fn guesswork(mut track: TrackInfo) -> TrackInfo {
	// Search for an artist name in the title
	const SEPARATORS: &[&str] = &[" - ", " – "];
	for sep in SEPARATORS {
		if track.title.contains(sep) {
			let mut parts = track.title.split(sep);
			track.artists = parts
				.next()
				.unwrap()
				.split(",")
				.map(|artist| artist.trim().to_owned())
				.collect();
			track.title = parts.next().unwrap().to_owned();
			return track;
		}
	}

	// Use channel name
	track
		.artists
		.get_mut(0)
		.map(|artist| *artist = artist.trim_end_matches(" - Topic").to_owned());
	track
}

impl YoutubeImportPlugin {
	pub fn import_impl(&self, url: &Url) -> Result<ImportedItem> {
		let endpoint = url
			.path_segments()
			.and_then(|s| s.last())
			.ok_or(anyhow!("couldn't find endpoint"))?;

		let rt = Runtime::new().unwrap();
		rt.block_on(async {
			Ok(match endpoint {
				"watch" => {
					let video_id: ytextract::video::Id = url
						.query_pairs()
						.find(|pair| pair.0 == "v")
						.ok_or(anyhow!("video id missing from the url"))?
						.1
						.parse()?;

					let video = self.client.video(video_id).await?;

					ImportedItem::Track(guesswork(tf_plugin::TrackInfo {
						url: Arc::new(url.clone()),
						artists: im::Vector::from_iter(once(video.channel().name().to_owned())),
						title: video.title().to_owned(),
					}))
				}
				"playlist" => {
					let playlist_id: ytextract::playlist::Id = url
						.query_pairs()
						.find(|pair| pair.0 == "list")
						.ok_or(anyhow!("playlist id missing from the url"))?
						.1
						.parse()?;

					let mut tracks = vec![];
					let videos = self.client.playlist(playlist_id).await?.videos();
					futures::pin_mut!(videos);

					while let Some(video) = videos.next().await {
						let video = video?;
						tracks.push(guesswork(tf_plugin::TrackInfo {
							url: Arc::new(
								format!("https://www.youtube.com/watch?v={}", video.id())
									.parse()
									.unwrap(),
							),
							artists: im::Vector::from_iter(once(video.channel().name().to_owned())),
							title: video.title().to_owned(),
						}))
					}
					ImportedItem::Playlist(tracks)
				}
				e => return Err(anyhow!("unknown endpoint: {}", e)),
			})
		})
	}
}

impl ImportPlugin for YoutubeImportPlugin {
	fn import(&mut self, url: &Url) -> Option<Result<ImportedItem>> {
		(url.scheme() == "https" && url.host_str() == Some("www.youtube.com"))
			.then(|| self.import_impl(url))
	}
}
