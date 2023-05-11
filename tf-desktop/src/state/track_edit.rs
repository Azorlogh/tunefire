use std::{collections::HashMap, rc::Rc};

use druid::{im, Data, Lens};
use tf_db::Track;
use uuid::Uuid;

use crate::widget::common::smart_list::IdentifiedVector;

#[derive(Clone, Data, Lens)]
pub struct TrackEdit {
	pub id: Rc<Uuid>,
	pub title: String,
	pub artists: IdentifiedVector<String>,
	pub source: String,
	pub tags: im::Vector<(u128, (String, f32))>,
	pub tag_suggestions: TagSuggestions,
}

#[derive(Clone, Data, Lens, Debug)]
pub struct TagSuggestions {
	pub tags: im::Vector<String>,
	pub selected: usize,
}

impl TrackEdit {
	pub fn new(id: Uuid, track: tf_db::Track) -> Self {
		Self {
			id: Rc::new(id),
			title: track.title,
			artists: im::Vector::from_iter(
				track.artists.into_iter().map(|name| (rand::random(), name)),
			),
			source: track.source,
			tags: im::Vector::from_iter(
				track
					.tags
					.iter()
					.map(|(n, v)| (rand::random(), (n.to_owned(), *v))),
			),
			tag_suggestions: TagSuggestions {
				tags: im::Vector::new(),
				selected: 0,
			},
		}
	}

	pub fn get_tags(&self) -> HashMap<String, f32> {
		self.tags.iter().map(|(_, t)| t).cloned().collect()
	}

	pub fn get_track(&self) -> Track {
		Track {
			source: self.source.clone(),
			artists: self.artists.iter().map(|(_, name)| name).cloned().collect(),
			title: self.title.clone(),
			tags: HashMap::from_iter(self.tags.iter().map(|(_, t)| t).cloned()),
		}
	}
}
