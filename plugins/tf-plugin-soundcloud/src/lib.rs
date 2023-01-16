use anyhow::{anyhow, Result};
use percent_encoding::{AsciiSet, CONTROLS};
use search::SoundcloudSearchPlugin;
use tf_plugin::{Plugin, SearchPlugin, SourcePlugin};
use tracing::debug;

mod api;
mod search;
mod source;

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

impl Plugin for Soundcloud {
	fn get_search_plugin(&self) -> Option<Box<dyn SearchPlugin>> {
		Some(Box::new(SoundcloudSearchPlugin {
			client_id: self.client_id.clone(),
		}))
	}

	fn get_source_plugin(&self) -> Option<Box<dyn SourcePlugin>> {
		Some(Box::new(source::SoundcloudSourcePlugin {
			client_id: self.client_id.clone(),
		}))
	}
}

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
