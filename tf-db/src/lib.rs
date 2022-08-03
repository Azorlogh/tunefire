use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::params;
use uuid::Uuid;

mod song;
pub use song::Song;

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

	pub fn add_song(&mut self, source: &str, title: &str) -> Result<Uuid> {
		let id = Uuid::new_v4();
		self.conn
			.execute(
				"INSERT INTO songs (id, source, title) VALUES (?, ?, ?)",
				&[&id.to_string(), source, title],
			)
			.map_err(|e| anyhow!("failed to add song {}", e))?;

		Ok(id)
	}

	pub fn delete_song(&mut self, id: Uuid) -> Result<()> {
		self.conn
			.execute("DELETE FROM songs WHERE id = ?", &[&id.to_string()])
			.map_err(|e| anyhow!("failed to delete song: {}", e))?;
		Ok(())
	}

	pub fn get_song(&self, id: Uuid) -> Result<Song> {
		let mut stmt = self
			.conn
			.prepare("SELECT id, source, title FROM songs WHERE id = ?")?;
		let mut song = stmt
			.query_row(&[&id.to_string()], |row| {
				Ok(Song {
					id: Uuid::try_parse(&row.get::<_, String>(0)?).unwrap(),
					source: row.get(1)?,
					title: row.get(2)?,
					tags: vec![],
				})
			})
			.map_err(|e| anyhow!("failed to get song: {}", e))?;

		let mut stmt = self.conn.prepare(
			r#"
			SELECT name, "value"
			FROM song_tags
			INNER JOIN songs ON song_tags.song_id = songs.id
			INNER JOIN tags ON song_tags.tag_id = tags.id
			WHERE songs.id = ?
		"#,
		)?;

		let tags = stmt
			.query_map(&[&id.to_string()], |row| Ok((row.get(0)?, row.get(1)?)))
			.unwrap();
		for tag in tags {
			song.tags.push(tag.unwrap());
		}

		Ok(song)
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

	pub fn set_tag(&mut self, song_id: Uuid, tag_name: &str, value: f32) -> Result<()> {
		if value == 0.0 {
			if let Ok(tag_id) = self.get_tag_id(tag_name) {
				self.conn.execute(
					"DELETE FROM song_tags WHERE song_id = ? AND tag_id = ?",
					params![&song_id.to_string(), &tag_id.to_string()],
				)?;
				self.conn.execute(
					"DELETE FROM tags WHERE id = ? AND id NOT IN (SELECT tag_id FROM song_tags)",
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
				"REPLACE INTO song_tags (song_id, tag_id, value) VALUES (?, ?, ?)",
				params![&song_id.to_string(), &tag_id.to_string(), &value],
			)
			.map_err(|e| anyhow!("failed to set tag: {}", e))?;
		Ok(())
	}

	pub fn list(&mut self) -> Result<Vec<Song>> {
		let mut stmt = self.conn.prepare("SELECT id, source, title FROM songs")?;
		let songs = stmt
			.query_map([], |row| {
				Ok(Song {
					id: Uuid::try_parse(&row.get::<_, String>(0)?).unwrap(),
					source: row.get(1)?,
					title: row.get(2)?,
					tags: vec![],
				})
			})?
			.collect::<Result<Vec<Song>, _>>()
			.map_err(|e| anyhow!("failed list songs: {}", e))?;

		Ok(songs)
	}

	// Create a view that contains all the songs that match the filter
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
								SELECT songs.id
								FROM songs
								LEFT JOIN song_tags ON song_tags.song_id = songs.id AND song_tags.tag_id = '{tag_id}'
								WHERE coalesce(song_tags.value, 0.0) {} {threshold}
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
						SELECT songs.id
						FROM songs
						WHERE songs.id NOT IN (SELECT id FROM "{f_uuid}")
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

	// Apply the filter to the list of songs.
	pub fn list_filtered(&mut self, filter: &Filter) -> Result<Vec<Song>> {
		println!("list filtered {:?}", filter);
		let view_id = self.view_filtered(filter)?;
		println!("obtained view id {:?}", view_id);
		let mut stmt = self.conn.prepare(&format!(
			r#"
				SELECT id, source, title
				FROM "{view_id}"
				NATURAL JOIN songs
			"#,
		))?;
		let songs = stmt
			.query_map(params![], |row| {
				Ok(Song {
					id: Uuid::try_parse(&row.get::<_, String>(0)?).unwrap(),
					source: row.get(1)?,
					title: row.get(2)?,
					tags: vec![],
				})
			})?
			.collect::<Result<Vec<Song>, _>>()
			.map_err(|e| anyhow!("failed list songs: {}", e))?;

		for view in self.query_views.drain(..) {
			self.conn.execute(&format!(r#"DROP VIEW "{view}""#), [])?;
		}

		Ok(songs)
	}
}
