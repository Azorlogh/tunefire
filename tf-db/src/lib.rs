use std::{
	collections::{HashMap, HashSet},
	io::Read,
	path::Path,
	str::FromStr,
};

use anyhow::{anyhow, Result};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use nom::AsBytes;
use uuid::Uuid;

mod data;
pub use data::{Playlist, Track};

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

	pub fn add_playlist(&mut self, playlist: &Playlist) -> Result<String> {
		println!("Adding {:?}", playlist);
		let serialized_playlist = serde_json::to_vec(&playlist)?;
		self.playlists
			.insert(playlist.name.to_owned(), serialized_playlist)?;
		Ok(playlist.name.to_owned())
	}

	// TODO
	pub fn set_playlist(&mut self, playlist_id: Uuid) -> Result<()> {
		Ok(())
	}

	pub fn delete_playlist(&mut self, name: &str) -> Result<()> {
		println!("Deleting playlist : {}", name);
		self.playlists.remove(name)?;
		Ok(())
	}

	pub fn iter_playlists(&mut self) -> impl Iterator<Item = Result<Playlist>> {
		self.playlists.iter().map(|kv| {
			let (_, playlist) = kv?;

			Ok(serde_json::from_slice::<Playlist>(&playlist.as_ref())?)
		})
	}

	pub fn add_track(&mut self, track: &Track) -> Result<Uuid> {
		// Check if track already exists, by title
		for v in self.iter_tracks() {
			let t = v?;

			if t.title == track.title {
				return Ok(t.id.unwrap());
			}
		}

		let id = Uuid::new_v4();
		let atrack = Track {
			id: Some(id),
			source: track.source.to_owned(),
			artists: track.artists.to_owned(),
			title: track.title.to_owned(),
			tags: track.tags.to_owned(),
		};
		let track = serde_json::to_vec(&atrack)?;
		self.tracks.insert(id, track)?;
		Ok(id)
	}

	pub fn set_track(&mut self, id: Uuid, track: &Track) -> Result<Uuid> {
		let track = serde_json::to_vec(&track)?;
		self.tracks.insert(id, track)?;
		Ok(id)
	}

	pub fn delete_track(&mut self, id: Uuid) -> Result<()> {
		self.tracks.remove(id)?;
		Ok(())
	}

	pub fn get_track(&self, id: Uuid) -> Result<Track> {
		Ok(serde_json::from_slice(
			self.tracks
				.get(id)?
				.ok_or(anyhow!("track `{id}` does not exist"))?
				.as_ref(),
		)?)
	}

	pub fn iter_tracks(&mut self) -> impl Iterator<Item = Result<Track>> {
		self.tracks.iter().map(|kv| {
			let (_, track) = kv?;
			Ok(serde_json::from_slice(track.as_ref())?)
		})
	}

	// Apply the filter to the list of tracks.
	pub fn list_filtered(&mut self, filter: &Filter) -> Result<Vec<Track>> {
		Ok(self
			.iter_tracks()
			.filter(|track| track.as_ref().map(|t| filter.matches(t)).unwrap_or(true))
			.collect::<Result<_>>()?)
	}

	pub fn get_tags(&mut self) -> Result<HashSet<String>> {
		let mut tags = HashSet::default();
		for t in self.iter_tracks() {
			for (tag_name, _) in &t?.tags {
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
