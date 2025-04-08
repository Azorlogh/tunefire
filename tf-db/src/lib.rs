use std::{collections::HashSet, io::Read, path::Path};

use anyhow::{anyhow, Result};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use nom::AsBytes;
use uuid::Uuid;

mod data;
pub use data::Track;

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
		// TODO artists ?

		Ok(Client {
			db,
			tracks,
			tags,
			playlists,
		})
	}

	pub fn add_playlist(&mut self, name: &str) -> Result<Uuid> {
		let mut playlist_id = Option::None;
		if self.playlists.iter().any(|kv| {
			let (id, playlist_name) = kv.expect("Could not get (key, value) pair of playlists.");
			playlist_id = Some(Uuid::from_slice(id.as_ref()));
			playlist_name == name
		}) {
			Ok(playlist_id.unwrap()?)
		} else {
			let id = Uuid::new_v4();
			self.db.open_tree(format!("{name}").into_bytes())?;
			self.playlists.insert(id, name)?;

			Ok(id)
		}
	}

	pub fn get_playlist_by_name(&mut self, name: &str) -> Result<Uuid> {
		let mut playlist_id = Option::None;
		if self.playlists.iter().any(|kv| {
			let (id, playlist_name) = kv.expect("Could not get (key, value) pair of playlists.");
			playlist_id = Some(Uuid::from_slice(id.as_ref()));
			playlist_name == name
		}) {
			Ok(playlist_id.unwrap()?)
		} else {
			Ok(Uuid::nil())
		}
	}

	// TODO
	pub fn get_playlist(&self, id: Uuid) -> Result<()> {
		let playlist_name = self.playlists.get(id)?;
		println!("Getting playlist : {:?}", playlist_name);

		Ok(())
	}

	pub fn delete_playlist(&mut self, id: Uuid) -> Result<()> {
		if let Some(playlist_name) = self.playlists.remove(id)? {
			self.db.drop_tree(playlist_name)?;
		}
		Ok(())
	}

	pub fn iter_playlists(&mut self) -> impl Iterator<Item = Result<(Uuid, String)>> {
		self.playlists.iter().map(|kv| {
			let (id, playlist_name) = kv?;
			Ok((
				Uuid::from_bytes(id.as_ref().try_into()?),
				std::str::from_utf8(&playlist_name).unwrap().to_string(),
			))
		})
	}

	pub fn add_track(&mut self, track: &Track) -> Result<Uuid> {
		let id = Uuid::new_v4();
		let track = serde_json::to_vec(track)?;
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

	pub fn iter_tracks(&mut self) -> impl Iterator<Item = Result<(Uuid, Track)>> {
		self.tracks.iter().map(|kv| {
			let (id, track) = kv?;
			Ok((
				Uuid::from_bytes(id.as_ref().try_into()?),
				serde_json::from_slice(track.as_ref())?,
			))
		})
	}

	// Apply the filter to the list of tracks.
	pub fn list_filtered(&mut self, filter: &Filter) -> Result<Vec<(Uuid, Track)>> {
		Ok(self
			.iter_tracks()
			.filter(|track| {
				track
					.as_ref()
					.map(|(_, t)| filter.matches(t))
					.unwrap_or(true)
			})
			.collect::<Result<_, _>>()?)
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
