use std::sync::Arc;

use anyhow::Result;

mod soundcloud;
use druid::{Data, ImageBuf, Lens};
pub use soundcloud::Soundcloud;
use tf_player::SourcePlugin;
use url::Url;

pub trait Plugin {
	fn get_search_plugin(&self) -> Option<Box<dyn SearchPlugin>>;

	fn get_source_plugin(&self) -> Option<Box<dyn SourcePlugin>>;
}

pub trait SearchPlugin: Send {
	fn search(&mut self, query: &str) -> Result<Vec<SearchResult>>;
}

#[derive(Debug, Clone, Data, Lens)]
pub struct SearchResult {
	pub url: Arc<Url>,
	pub artist: String,
	pub title: String,
	pub artwork: Option<ImageBuf>,
}
