use anyhow::Result;
use uuid::Uuid;

use crate::Client;

impl Client {
	pub fn set_tag(&mut self, id: Uuid, tag_name: &str, value: f32) -> Result<()> {
		let mut track = self.get_track(id)?;
		track.tags.insert(tag_name.to_owned(), value);
		self.set_track(id, &track)?;
		Ok(())
	}
}
