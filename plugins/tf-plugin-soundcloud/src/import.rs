use std::{iter::once, sync::Arc};

use anyhow::Result;
use druid::im;
use tf_plugin::{ImportPlugin, ImportedItem, TrackInfo};
use url::Url;

use crate::client::{Client, ResolvedItem};

pub struct SoundcloudImportPlugin {
	pub client: Client,
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

impl SoundcloudImportPlugin {
	pub fn import_impl(&self, url: &Url) -> Result<ImportedItem> {
		let item = self.client.resolve(url)?;
		Ok(match item {
			ResolvedItem::Track(track) => ImportedItem::Track(guesswork(tf_plugin::TrackInfo {
				url: Arc::new(url.clone()),
				artists: im::Vector::from_iter(once(track.user.username)),
				title: track.title,
			})),
			ResolvedItem::Playlist(playlist) => ImportedItem::Playlist(
				playlist
					.tracks
					.iter()
					.map(|track| {
						guesswork(tf_plugin::TrackInfo {
							url: Arc::new(track.permalink_url.clone()),
							artists: im::Vector::from_iter(once(track.user.username.clone())),
							title: track.title.clone(),
						})
					})
					.collect(),
			),
		})
	}
}

impl ImportPlugin for SoundcloudImportPlugin {
	fn import(&mut self, url: &Url) -> Option<Result<ImportedItem>> {
		(url.scheme() == "https" && url.host_str() == Some("soundcloud.com"))
			.then(|| self.import_impl(url))
	}
}
