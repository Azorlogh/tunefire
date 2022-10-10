use anyhow::{anyhow, Result};
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
		let response: api::SearchResponse = serde_json::from_str(
			&ureq::get(&format!(
				"https://api-v2.soundcloud.com/search?client_id={}&q={}&limit=10&offset=0&linked_partitioning=1&app_version=1665395834&app_locale=en",
				self.client_id, query_enc
			))
			.call()?
			.into_string()?,
		)?;

		Ok(response
			.collection
			.into_iter()
			.map(|res| super::SearchResult {
				url: res.permalink_url,
				artist: res.user.username,
				title: res.title,
				artwork: res.artwork_url,
			})
			.collect())
	}
}
