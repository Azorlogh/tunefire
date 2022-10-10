use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize)]
pub struct SearchResponse {
	pub collection: Vec<SearchResult>,
}

#[derive(Deserialize, Serialize)]
pub struct SearchResult {
	pub permalink_url: Url,
	pub user: User,
	pub title: String,
	pub artwork_url: Url,
}

#[derive(Deserialize, Serialize)]
pub struct User {
	pub username: String,
}
