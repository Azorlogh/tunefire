use std::{
	collections::{HashMap, HashSet},
	path::Path,
};

use anyhow::{anyhow, Result};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use uuid::Uuid;

mod data;
pub use data::Track;

mod filter;
pub use filter::Filter;

mod tags;

#[derive(Debug)]
pub struct Client {
	pub db: sled::Db,
	pub tracks: sled::Tree,
	pub tags: sled::Tree,
}

impl Client {
	pub fn new<P>(path: P) -> Result<Self>
	where
		P: AsRef<Path>,
	{
		let db = sled::open(path)?;
		let tracks = db.open_tree(b"tracks")?;
		let tags = db.open_tree(b"tags")?;

		Ok(Client { db, tracks, tags })
	}

	pub fn add_track(&mut self, source: &str, artist: &str, title: &str) -> Result<Uuid> {
		let id = Uuid::new_v4();
		let track = serde_json::to_vec(&Track {
			source: source.to_owned(),
			artist: artist.to_owned(),
			title: title.to_owned(),
			tags: HashMap::default(),
		})?;
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
