use std::path::Path;

use anyhow::{anyhow, Result};
use data::Tag;
use rusqlite::params;
use uuid::Uuid;

mod data;
pub use data::Track;

mod filter;
pub use filter::Filter;

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
					tags: vec![],
				})
			})
			.map_err(|e| anyhow!("failed to get track: {}", e))?;

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
		for tag in tags {
			track.tags.push(tag.unwrap());
		}

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

	pub fn set_tag(&mut self, track_id: Uuid, tag_name: &str, value: f32) -> Result<()> {
		if value == 0.0 {
			if let Ok(tag_id) = self.get_tag_id(tag_name) {
				self.conn.execute(
					"DELETE FROM track_tags WHERE track_id = ? AND tag_id = ?",
					params![&track_id.to_string(), &tag_id.to_string()],
				)?;
				self.conn.execute(
					"DELETE FROM tags WHERE id = ? AND id NOT IN (SELECT tag_id FROM track_tags)",
					[&tag_id.to_string()],
				)?;
			}
			return Ok(());
		}

		let tag_id = match self.get_tag_id(tag_name) {
			Ok(tag_id) => tag_id,
			Err(_) => {
				let tag_id = Uuid::new_v4();
				self.conn
					.execute(
						"INSERT INTO tags (id, name) VALUES (?, ?)",
						params![tag_id.to_string(), tag_name],
					)
					.map_err(|e| anyhow!("failed to add tag: {}", e))?;
				tag_id
			}
		};

		self.conn
			.execute(
				"REPLACE INTO track_tags (track_id, tag_id, value) VALUES (?, ?, ?)",
				params![&track_id.to_string(), &tag_id.to_string(), &value],
			)
			.map_err(|e| anyhow!("failed to set tag: {}", e))?;
		Ok(())
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
					tags: vec![],
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
		println!("list filtered {:?}", filter);
		let view_id = self.view_filtered(filter)?;
		println!("obtained view id {:?}", view_id);
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
					tags: vec![],
				})
			})?
			.collect::<Result<Vec<Track>, _>>()
			.map_err(|e| anyhow!("failed list tracks: {}", e))?;

		for view in self.query_views.drain(..) {
			self.conn.execute(&format!(r#"DROP VIEW "{view}""#), [])?;
		}

		Ok(tracks)
	}

	// List the set of tags
	pub fn list_tags(&mut self) -> Result<Vec<Tag>> {
		let mut stmt = self.conn.prepare(&format!(
			r#"
				SELECT id, name
				FROM tags
				ORDER BY name
			"#,
		))?;
		let tags = stmt
			.query_map(params![], |row| {
				Ok(Tag {
					id: Uuid::try_parse(&row.get::<_, String>(0)?).unwrap(),
					name: row.get(1)?,
				})
			})?
			.collect::<Result<Vec<Tag>, _>>()
			.map_err(|e| anyhow!("failed list tags: {}", e))?;
		Ok(tags)
	}
}
