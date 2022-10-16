use std::rc::Rc;

use anyhow::Result;

mod soundcloud;
use druid::{Data, ImageBuf, Lens};
pub use soundcloud::Soundcloud;
use url::Url;

pub trait Plugin {
	fn search(&self, query: &str) -> Result<Vec<SearchResult>>;
}

#[derive(Debug, Clone, Data, Lens)]
pub struct SearchResult {
	pub url: Rc<Url>,
	pub artist: String,
	pub title: String,
	// pub artwork: Rc<Url>,
	pub artwork: ImageBuf,
}
