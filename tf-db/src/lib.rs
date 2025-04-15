use std::{
	collections::{HashMap, HashSet},
	io::{Read, WriterPanicked},
	path::Path,
	str::FromStr,
};

use anyhow::{anyhow, Result};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use nom::AsBytes;
use uuid::Uuid;

#[macro_use]
mod utils;

mod data;
pub use data::{Playlist, PlaylistId, TagId, Track, TrackId};

mod filter;
pub use filter::Filter;

mod tags;

#[derive(Debug, Clone)]
pub struct Client {
	pub db: sled::Db,
	pub tracks: sled::Tree,
	pub tags: sled::Tree,
	pub playlists: sled::Tree,
}

impl Client {
	pub fn new<P>(path: P) -> Result<Self>
	where
		P: AsRef<Path>,
	{
		let db = sled::open(path)?;
		let tracks = db.open_tree(b"tracks")?;
		let tags = db.open_tree(b"tags")?;
		let playlists = db.open_tree(b"playlists")?;

		Ok(Client {
			db,
			tracks,
			tags,
			playlists,
		})
	}

	pub fn add_playlist(&mut self, playlist: &Playlist) -> Result<PlaylistId> {
		println!("Adding {:?}", playlist);
		// Check if track already exists, by title

		for kv in self.iter_playlists() {
			let (id, p) = kv?;
			if p.name == playlist.name {
				println!("Playlist already exists !");
				return Ok(id);
			}
		}

		let id = PlaylistId::new();

		let serialized_playlist = serde_json::to_vec(&playlist)?;
		self.playlists.insert(id, serialized_playlist)?;
		Ok(id)
	}

	pub fn set_playlist(&mut self, id: PlaylistId, playlist: &Playlist) -> Result<PlaylistId> {
		let p = serde_json::to_vec(&playlist)?;
		self.playlists.insert(id, p)?;
		Ok(id)
	}

	pub fn add_track_to_playlist(
		&mut self,
		playlist_id: PlaylistId,
		track_id: TrackId,
	) -> Result<()> {
		let mut playlist = self.get_playlist(playlist_id)?;

		playlist.track_ids.push(track_id);

		self.set_playlist(playlist_id, &playlist)?;

		Ok(())
	}

	pub fn delete_track_from_playlist(
		&mut self,
		playlist_id: PlaylistId,
		track_id: TrackId,
	) -> Result<()> {
		let mut playlist = self.get_playlist(playlist_id)?;

		if let Some(index) = playlist.track_ids.iter().position(|id| *id == track_id) {
			playlist.track_ids.swap_remove(index);
		}

		self.set_playlist(playlist_id, &playlist)?;

		Ok(())
	}

	pub fn get_playlist(&self, id: PlaylistId) -> Result<Playlist> {
		Ok(serde_json::from_slice(
			self.playlists
				.get(id)?
				.ok_or(anyhow!("playlist `{id}` does not exist"))?
				.as_ref(),
		)?)
	}

	pub fn delete_playlist(&mut self, id: PlaylistId) -> Result<()> {
		println!("Deleting playlist : {}", id);
		self.playlists.remove(id)?;
		Ok(())
	}

	pub fn iter_playlists(&mut self) -> impl Iterator<Item = Result<(PlaylistId, Playlist)>> {
		self.playlists.iter().map(|kv| {
			let (id, playlist) = kv?;

			Ok((
				PlaylistId::try_from(id.as_ref())?,
				serde_json::from_slice::<Playlist>(&playlist.as_ref())?,
			))
		})
	}

	pub fn add_track(&mut self, track: &Track) -> Result<TrackId> {
		let id = TrackId::new();
		let track = serde_json::to_vec(&track)?;
		self.tracks.insert(id, track)?;
		Ok(id)
	}

	pub fn set_track(&mut self, id: TrackId, track: &Track) -> Result<TrackId> {
		let track = serde_json::to_vec(&track)?;
		self.tracks.insert(id, track)?;
		Ok(id)
	}

	pub fn delete_track(&mut self, id: TrackId) -> Result<()> {
		self.tracks.remove(id)?;
		Ok(())
	}

	pub fn get_track(&self, id: TrackId) -> Result<Track> {
		Ok(serde_json::from_slice(
			self.tracks
				.get(id)?
				.ok_or(anyhow!("track `{id}` does not exist"))?
				.as_ref(),
		)?)
	}

	pub fn iter_tracks(&mut self) -> impl Iterator<Item = Result<(TrackId, Track)>> {
		self.tracks.iter().map(|kv| {
			let (id, track) = kv?;
			Ok((
				TrackId::try_from(id.as_ref())?,
				serde_json::from_slice(track.as_ref())?,
			))
		})
	}

	// Apply the filter to the list of tracks.
	pub fn list_filtered(&mut self, filter: &Filter) -> Result<Vec<(TrackId, Track)>> {
		Ok(self
			.iter_tracks()
			.filter(|track| {
				track
					.as_ref()
					.map(|(_, t)| filter.matches(t))
					.unwrap_or(true)
			})
			.collect::<Result<_>>()?)
	}

	pub fn get_tags(&mut self) -> Result<HashSet<String>> {
		let mut tags = HashSet::default();
		for t in self.iter_tracks() {
			for (tag_name, _) in &t?.1.tags {
				tags.insert(tag_name.to_owned());
			}
		}
		Ok(tags)
	}

	pub fn search_tag(&mut self, q: &str, limit: usize) -> Result<Vec<(String, Vec<usize>)>> {
		let matcher = SkimMatcherV2::default();
		let mut matches = self
			.get_tags()?
			.into_iter()
			.filter_map(|tag| Some((matcher.fuzzy_indices(&tag, q)?, tag)))
			.collect::<Vec<_>>();
		matches.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));
		Ok(matches
			.into_iter()
			.take(limit)
			.map(|((_, indices), tag)| (tag, indices))
			.collect())
	}
}
