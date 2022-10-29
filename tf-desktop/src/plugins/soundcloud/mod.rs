use std::rc::Rc;

use anyhow::{anyhow, Result};
use druid::piet::ImageFormat;
use percent_encoding::{AsciiSet, CONTROLS};
use tracing::debug;

use super::Plugin;

mod api;

pub struct Soundcloud {
	client_id: String,
}

impl Soundcloud {
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
}

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

impl Plugin for Soundcloud {
	fn search(&self, query: &str) -> Result<Vec<super::SearchResult>> {
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
					Some(super::SearchResult {
						url: Rc::new(permalink_url),
						artist: user.username,
						title,
						artwork,
					})
				}
				_ => None,
			})
			.collect())
	}
}
