use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize)]
pub struct SearchResponse {
	pub collection: Vec<SearchResult>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "kind")]
pub enum SearchResult {
	Track {
		permalink_url: Url,
		user: User,
		title: String,
		artwork_url: Option<Url>,
	},
	User,
	Playlist,
}

#[derive(Deserialize, Serialize)]
pub struct User {
	pub username: String,
}
