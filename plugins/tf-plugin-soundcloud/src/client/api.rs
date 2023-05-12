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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "kind")]
pub enum ResolveResponse {
	Track(ResolvedTrack),
	Playlist(ResolvedPlaylist),
}

#[derive(Deserialize, Serialize)]
pub struct Track {
	pub id: u64,
}

#[derive(Deserialize, Serialize)]
pub struct ResolvedTrack {
	pub id: u64,
	pub permalink_url: Url,
	pub track_authorization: String,
	pub media: Media,
	pub user: User,
	pub title: String,
}

#[derive(Default, Deserialize, Serialize)]
pub struct ResolvedPlaylist {
	pub tracks: Vec<Track>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Media {
	pub transcodings: Vec<Transcoding>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transcoding {
	pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MediaResponse {
	pub url: String,
}
