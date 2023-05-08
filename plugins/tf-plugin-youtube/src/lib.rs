use anyhow::Result;
use tf_plugin::{ImportPlugin, Plugin, SourcePlugin};

mod import;
mod source;

pub struct Youtube {
	client: ytextract::Client,
}

impl Youtube {
	pub fn new() -> Result<Self> {
		Ok(Self {
			client: ytextract::Client::new(),
		})
	}
}

impl Plugin for Youtube {
	fn get_source_plugin(&self) -> Option<Box<dyn SourcePlugin>> {
		Some(Box::new(source::YoutubeSourcePlugin {
			client: self.client.clone(),
		}))
	}

	fn get_import_plugin(&self) -> Option<Box<dyn ImportPlugin>> {
		Some(Box::new(import::YoutubeImportPlugin {
			client: self.client.clone(),
		}))
	}
}
