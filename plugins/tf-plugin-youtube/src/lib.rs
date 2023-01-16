use anyhow::Result;
use tf_plugin::{Plugin, SourcePlugin};

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
}
