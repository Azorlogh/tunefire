use std::collections::HashMap;

use anyhow::{anyhow, Result};
use rusqlite::params;
use uuid::Uuid;

use crate::{data::Tag, Client};

impl Client {
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

	pub fn set_tags(&mut self, id: Uuid, new_tags: &HashMap<String, f32>) -> Result<()> {
		let track = self.get_track(id)?;
		for (tag_name, _) in &track.tags {
			if !new_tags.contains_key(tag_name) {
				self.set_tag(id, tag_name, 0.0)?;
			}
		}
		for (tag_name, tag_value) in new_tags {
			self.set_tag(id, tag_name, *tag_value)?;
		}
		Ok(())
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
