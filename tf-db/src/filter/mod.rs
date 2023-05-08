use std::{collections::HashSet, iter::once};

use crate::Track;

mod parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
	All,
	LessThan {
		tag: String,
		threshold: f32,
		inclusive: bool,
	},
	Artist(String),
	And(Box<Filter>, Box<Filter>),
	Or(Box<Filter>, Box<Filter>),
	Not(Box<Filter>),
}

impl Filter {
	pub fn get_tag_set(&self) -> HashSet<String> {
		match self {
			Filter::All => HashSet::default(),
			Filter::LessThan { tag, .. } => once(tag.clone()).collect(),
			Filter::Artist(_) => HashSet::default(),
			Filter::And(f0, f1) => f0.get_tag_set().union(&f1.get_tag_set()).cloned().collect(),
			Filter::Or(f0, f1) => f0.get_tag_set().union(&f1.get_tag_set()).cloned().collect(),
			Filter::Not(f) => f.get_tag_set(),
		}
	}

	pub fn matches(&self, track: &Track) -> bool {
		match self {
			Filter::All => true,
			Filter::LessThan {
				tag,
				threshold,
				inclusive,
			} => {
				let Some(value) = track.tags.get(tag) else { return false; };
				if *inclusive {
					value <= threshold
				} else {
					value < threshold
				}
			}
			Filter::Artist(artist) => track.artist == *artist,
			Filter::And(f0, f1) => f0.matches(track) && f1.matches(track),
			Filter::Or(f0, f1) => f0.matches(track) && f1.matches(track),
			Filter::Not(f) => !f.matches(track),
		}
	}
}
