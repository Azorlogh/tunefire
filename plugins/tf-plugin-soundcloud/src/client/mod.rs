use anyhow::Result;
use url::Url;

pub mod api;

pub enum ResolvedItem {
	Track(api::ResolvedTrack),
	Playlist(ResolvedPlaylist),
}

pub struct ResolvedPlaylist {
	pub tracks: Vec<api::ResolvedTrack>,
}

pub struct Client {
	pub client_id: String,
}

impl Client {
	pub fn resolve(&self, url: &Url) -> Result<ResolvedItem> {
		let resolved: api::ResolveResponse = serde_json::from_str(
			&ureq::get(&format!(
				"https://api-v2.soundcloud.com/resolve?client_id={}&url={}",
				self.client_id, url
			))
			.call()?
			.into_string()?,
		)?;
		Ok(match resolved {
			api::ResolveResponse::Track(track) => ResolvedItem::Track(track),
			api::ResolveResponse::Playlist(playlist) => {
				let mut tracks = vec![];
				for tr in playlist.tracks.chunks(50) {
					tracks.extend(serde_json::from_str::<Vec<api::ResolvedTrack>>(
						&ureq::get(&format!(
							"https://api-v2.soundcloud.com/tracks?client_id={}&ids={}",
							self.client_id,
							tr.iter()
								.map(|t| t.id.to_string())
								.collect::<Vec<_>>()
								.join(",")
						))
						.call()?
						.into_string()?,
					)?);
				}
				ResolvedItem::Playlist(ResolvedPlaylist { tracks })
			}
		})
	}
}
