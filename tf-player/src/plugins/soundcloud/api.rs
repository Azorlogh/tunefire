use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ResolveResponse {
	pub track_authorization: String,
	pub media: Media,
}

#[derive(Debug, Deserialize, Serialize)]
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
