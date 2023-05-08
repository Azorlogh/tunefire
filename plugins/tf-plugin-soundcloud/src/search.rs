use std::{iter::once, sync::Arc};

use anyhow::Result;
use druid::{im, piet::ImageFormat};
use tf_plugin::SearchPlugin;

use crate::{api, FRAGMENT};

pub struct SoundcloudSearchPlugin {
	pub client_id: String,
}

impl SearchPlugin for SoundcloudSearchPlugin {
	fn search(&mut self, query: &str) -> Result<Vec<tf_plugin::SearchResult>> {
		let query_enc = percent_encoding::utf8_percent_encode(query, FRAGMENT);
		let response_json = ureq::get(&format!(
			"https://api-v2.soundcloud.com/search?client_id={}&q={}&limit=10&offset=0&linked_partitioning=1&app_version=1665395834&app_locale=en",
			self.client_id, query_enc
		))
		.call()?
		.into_string()?;
		let response: api::SearchResponse = serde_json::from_str(&response_json)?;

		Ok(response
			.collection
			.into_iter()
			.filter_map(|res| match res {
				api::SearchResult::Track {
					permalink_url,
					user,
					title,
					artwork_url,
				} => {
					let artwork = artwork_url.and_then(|url| {
						let mut artwork_buf = vec![];
						ureq::get(url.as_str())
							.call()
							.ok()?
							.into_reader()
							.read_to_end(&mut artwork_buf)
							.ok()?;
						let artwork_image =
							image::io::Reader::new(std::io::Cursor::new(artwork_buf))
								.with_guessed_format()
								.ok()?
								.decode()
								.ok()?
								.to_rgb8();
						Some(druid::ImageBuf::from_raw(
							artwork_image.as_raw().as_slice(),
							ImageFormat::Rgb,
							artwork_image.width() as usize,
							artwork_image.height() as usize,
						))
					});
					Some(tf_plugin::SearchResult {
						url: Arc::new(permalink_url),
						artist: im::Vector::from_iter(once(user.username)),
						title,
						artwork,
					})
				}
				_ => None,
			})
			.collect())
	}
}
