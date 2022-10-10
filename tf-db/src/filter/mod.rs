use std::{collections::HashSet, iter::once};

mod parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
	And(Box<Filter>, Box<Filter>),
	Or(Box<Filter>, Box<Filter>),
	Not(Box<Filter>),
	LessThan {
		tag: String,
		threshold: f32,
		inclusive: bool,
	},
}

impl Filter {
	pub fn get_tag_set(&self) -> HashSet<String> {
		match self {
			Filter::LessThan { tag, .. } => once(tag.clone()).collect(),
			Filter::And(f0, f1) => f0.get_tag_set().union(&f1.get_tag_set()).cloned().collect(),
			Filter::Or(f0, f1) => f0.get_tag_set().union(&f1.get_tag_set()).cloned().collect(),
			Filter::Not(f) => f.get_tag_set(),
		}
	}
}