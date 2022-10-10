use std::{collections::HashMap, rc::Rc};

use druid::{im, Data, Lens};
use uuid::Uuid;

#[derive(Clone, Data, Lens)]
pub struct TrackEdit {
	pub id: Rc<Uuid>,
	pub title: String,
	pub source: String,
	pub tags: im::Vector<(String, f32)>,
	pub tag_suggestions: TagSuggestions,
}

#[derive(Clone, Data, Lens, Debug)]
pub struct TagSuggestions {
	pub tags: im::Vector<String>,
	pub selected: usize,
}

impl TrackEdit {
	pub fn new(track: tf_db::Track) -> Self {
		Self {
			id: Rc::new(track.id),
			title: track.title,
			source: track.source,
			tags: im::Vector::from_iter(track.tags.clone()),
			tag_suggestions: TagSuggestions {
				tags: im::Vector::new(),
				selected: 0,
			},
		}
	}

	pub fn get_tags(&self) -> HashMap<String, f32> {
		self.tags
			.iter()
			.filter(|t| !t.0.is_empty())
			.cloned()
			.collect()
	}
}
