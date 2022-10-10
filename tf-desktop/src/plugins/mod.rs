use anyhow::Result;

mod soundcloud;
pub use soundcloud::Soundcloud;
use url::Url;

pub trait Plugin {
	fn search(&self, query: &str) -> Result<Vec<SearchResult>>;
}

pub struct SearchResult {
	pub url: Url,
	pub artist: String,
	pub title: String,
	pub artwork: Url,
}
