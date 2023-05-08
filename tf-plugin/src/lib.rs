use std::sync::Arc;

use anyhow::Result;
use druid::{im, Data, ImageBuf, Lens};
pub use tf_player::{self as player, SourcePlugin};
use url::Url;

pub trait Plugin {
	fn get_search_plugin(&self) -> Option<Box<dyn SearchPlugin>> {
		None
	}

	fn get_source_plugin(&self) -> Option<Box<dyn SourcePlugin>> {
		None
	}

	fn get_import_plugin(&self) -> Option<Box<dyn ImportPlugin>> {
		None
	}
}

pub trait SearchPlugin: Send {
	fn search(&mut self, query: &str) -> Result<Vec<SearchResult>>;
}

#[derive(Debug, Clone, Data, Lens)]
pub struct SearchResult {
	pub url: Arc<Url>,
	pub artists: im::Vector<String>,
	pub title: String,
	pub artwork: Option<ImageBuf>,
}

pub trait ImportPlugin: Send {
	fn import(&mut self, url: &Url) -> Option<Result<ImportedItem>>;
}

pub enum ImportedItem {
	Track(TrackInfo),
	Set(Vec<TrackInfo>),
}

pub struct TrackInfo {
	pub url: Arc<Url>,
	pub artists: im::Vector<String>,
	pub title: String,
}
