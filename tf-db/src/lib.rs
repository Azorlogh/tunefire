use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, Result};
use rusqlite::params;
use uuid::Uuid;

mod data;
pub use data::Track;

mod filter;
pub use filter::Filter;

mod tags;

#[derive(Debug)]
pub struct Client {
	pub conn: rusqlite::Connection,
	pub query_views: Vec<Uuid>,
}

const MOCKUP: bool = false;

impl Client {
	pub fn new<P>(path: P) -> Result<Self>
	where
		P: AsRef<Path>,
	{
		let conn = rusqlite::Connection::open(path)
			.map_err(|e| anyhow!("failed to open the database: {}", e))?;

		if MOCKUP {
			conn.execute_batch(include_str!("cleanup.sql"))
				.map_err(|e| anyhow!("failed to initialize database: {}", e))?;
		}

		conn.execute_batch(include_str!("schema.sql"))
			.map_err(|e| anyhow!("failed to initialize database: {}", e))?;

		if MOCKUP {
			conn.execute_batch(include_str!("mockup.sql"))
				.map_err(|e| anyhow!("failed to initialize database: {}", e))?;
		}

		Ok(Client {
			conn,
			query_views: vec![],
		})
	}

	pub fn add_track(&mut self, source: &str, artist: &str, title: &str) -> Result<Uuid> {
		let id = Uuid::new_v4();
		self.conn
			.execute(
				"INSERT INTO tracks (id, source, artist, title) VALUES (?, ?, ?, ?)",
				&[&id.to_string(), source, artist, title],
			)
			.map_err(|e| anyhow!("failed to add track {}", e))?;

		Ok(id)
	}

	pub fn delete_track(&mut self, id: Uuid) -> Result<()> {
		self.conn
			.execute("DELETE FROM tracks WHERE id = ?", &[&id.to_string()])
			.map_err(|e| anyhow!("failed to delete track: {}", e))?;
		Ok(())
	}

	pub fn get_track_tags(&self, id: Uuid) -> Result<HashMap<String, f32>> {
		let mut stmt = self.conn.prepare(
			r#"
			SELECT name, "value"
			FROM track_tags
			INNER JOIN tracks ON track_tags.track_id = tracks.id
			INNER JOIN tags ON track_tags.tag_id = tags.id
			WHERE tracks.id = ?
		"#,
		)?;

		let tags = stmt
			.query_map(&[&id.to_string()], |row| Ok((row.get(0)?, row.get(1)?)))
			.unwrap();
		let mut result = HashMap::new();
		for tag in tags.into_iter() {
			let (name, value) = tag.unwrap();
			result.insert(name, value);
		}
		Ok(result)
	}

	pub fn get_track(&self, id: Uuid) -> Result<Track> {
		let mut stmt = self
			.conn
			.prepare("SELECT id, source, artist, title FROM tracks WHERE id = ?")?;
		let mut track = stmt
			.query_row(&[&id.to_string()], |row| {
				Ok(Track {
					id: Uuid::try_parse(&row.get::<_, String>(0)?).unwrap(),
					source: row.get(1)?,
					artist: row.get(2)?,
					title: row.get(3)?,
					tags: HashMap::new(),
				})
			})
			.map_err(|e| anyhow!("failed to get track: {}", e))?;
		track.tags = self.get_track_tags(track.id)?;

		Ok(track)
	}

	fn get_tag_id(&self, name: &str) -> Result<Uuid> {
		let mut stmt = self.conn.prepare("SELECT id FROM tags WHERE name = ?")?;
		let id = stmt
			.query_row(&[name], |row| {
				Ok(Uuid::try_parse(&row.get::<_, String>(0)?).unwrap())
			})
			.map_err(|e| anyhow!("failed to get tag id: {}", e))?;
		Ok(id)
	}

	pub fn list(&mut self) -> Result<Vec<Track>> {
		let mut stmt = self
			.conn
			.prepare("SELECT id, source, artist, title FROM tracks")?;
		let tracks = stmt
			.query_map([], |row| {
				Ok(Track {
					id: Uuid::try_parse(&row.get::<_, String>(0)?).unwrap(),
					source: row.get(1)?,
					artist: row.get(2)?,
					title: row.get(3)?,
					tags: HashMap::new(),
				})
			})?
			.collect::<Result<Vec<Track>, _>>()
			.map_err(|e| anyhow!("failed list tracks: {}", e))?;

		Ok(tracks)
	}

	// Create a view that contains all the tracks that match the filter
	fn view_filtered(&mut self, filter: &Filter) -> Result<Uuid> {
		let uuid = Uuid::new_v4();
		let uuid = match filter {
			Filter::All => {
				self.conn.execute(
					&format!(
						r#"
						CREATE TEMP VIEW "{uuid}" AS
						SELECT tracks.id
						FROM tracks
						"#,
					),
					[],
				)?;
				uuid
			}
			Filter::LessThan {
				tag,
				threshold,
				inclusive,
			} => {
				let tag_id = self.get_tag_id(tag).unwrap_or(Uuid::nil());
				self.conn.execute(
					&format!(
						r#"
						CREATE TEMP VIEW "{uuid}" AS
						SELECT tracks.id
						FROM tracks
						LEFT JOIN track_tags ON track_tags.track_id = tracks.id AND track_tags.tag_id = '{tag_id}'
						WHERE coalesce(track_tags.value, 0.0) {} {threshold}
						"#,
						if *inclusive { "<=" } else { "<" },
					),
					[],
				)?;
				uuid
			}
			Filter::Artist(artist) => {
				self.conn.execute(
					&format!(
						r#"
						CREATE TEMP VIEW "{uuid}" AS
						SELECT tracks.id
						FROM tracks
						WHERE tracks.artist = '{artist}'
						"#,
					),
					[],
				)?;
				uuid
			}
			Filter::Not(f) => {
				let f_uuid = self.view_filtered(f)?;
				self.conn.execute(
					&format!(
						r#"
						CREATE TEMP VIEW "{uuid}" AS
						SELECT tracks.id
						FROM tracks
						WHERE tracks.id NOT IN (SELECT id FROM "{f_uuid}")
						"#,
					),
					[],
				)?;
				uuid
			}
			Filter::And(fa, fb) => {
				let fa_uuid = self.view_filtered(fa)?;
				let fb_uuid = self.view_filtered(fb)?;
				self.conn.execute(
					&format!(
						r#"
						CREATE TEMP VIEW "{uuid}" AS
						SELECT a.id
						FROM "{fa_uuid}" AS a
						INNER JOIN "{fb_uuid}" AS b ON a.id = b.id
						"#,
					),
					[],
				)?;
				uuid
			}
			Filter::Or(fa, fb) => {
				let fa_uuid = self.view_filtered(fa)?;
				let fb_uuid = self.view_filtered(fb)?;
				self.conn.execute(
					&format!(
						r#"
						CREATE TEMP VIEW "{uuid}" AS
						SELECT a.id
						FROM "{fa_uuid}" AS a
						UNION
						SELECT b.id
						FROM "{fb_uuid}" AS b
						"#,
					),
					[],
				)?;
				uuid
			}
		};
		self.query_views.push(uuid);
		Ok(uuid)
	}

	// Apply the filter to the list of tracks.
	pub fn list_filtered(&mut self, filter: &Filter) -> Result<Vec<Track>> {
		let view_id = self.view_filtered(filter)?;
		let mut stmt = self.conn.prepare(&format!(
			r#"
				SELECT id, source, artist, title
				FROM "{view_id}"
				NATURAL JOIN tracks
			"#,
		))?;
		let tracks = stmt
			.query_map(params![], |row| {
				Ok(Track {
					id: Uuid::try_parse(&row.get::<_, String>(0)?).unwrap(),
					source: row.get(1)?,
					artist: row.get(2)?,
					title: row.get(3)?,
					tags: HashMap::new(),
				})
			})?
			.map(|result| {
				result.map(|mut track| {
					track.tags = self.get_track_tags(track.id).unwrap();
					track
				})
			})
			.collect::<Result<Vec<Track>, _>>()
			.map_err(|e| anyhow!("failed list tracks: {}", e))?;

		for view in self.query_views.drain(..) {
			self.conn.execute(&format!(r#"DROP VIEW "{view}""#), [])?;
		}

		Ok(tracks)
	}

	pub fn search_tag(&mut self, q: &str) -> Result<Vec<String>> {
		let mut stmt = self.conn.prepare(&format!(
			r#"
				SELECT name FROM tag_search('{q}');
			"#
		))?;
		let tags = stmt
			.query_map(params![], |row| Ok(row.get::<_, String>(0)?))?
			.collect::<Result<Vec<String>, _>>()
			.map_err(|e| anyhow!("failed to find tags: {}", e))?;
		Ok(tags)
	}
}
