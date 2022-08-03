use std::time::Duration;

use anyhow::{anyhow, Result};

use crate::SongInfo;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
	Idle,
	Playing(Playing),
}
use State::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Playing {
	pub song: SongInfo,
	pub offset: Duration,
	pub paused: bool,
}

impl Default for State {
	fn default() -> Self {
		Idle
	}
}

impl State {
	pub fn get_playing(&self) -> Option<&Playing> {
		match self {
			Playing(s) => Some(s),
			_ => None,
		}
	}

	pub fn current_song(&self) -> Option<&SongInfo> {
		match self {
			Playing(Playing { song, .. }) => Some(song),
			_ => None,
		}
	}

	pub fn current_time(&self) -> Option<&Duration> {
		match self {
			Playing(Playing { offset, .. }) => Some(offset),
			_ => None,
		}
	}

	pub(super) fn set_song(&mut self, song: SongInfo) {
		*self = Playing(Playing {
			song,
			offset: Duration::ZERO,
			paused: false,
		});
	}

	pub(super) fn play(&mut self) -> Result<()> {
		match self {
			Playing(Playing { paused, .. }) => {
				*paused = false;
				Ok(())
			}
			_ => Err(anyhow!("wrong state")),
		}
	}

	pub(super) fn pause(&mut self) -> Result<()> {
		match self {
			Playing(Playing { paused, .. }) => {
				*paused = true;
				Ok(())
			}
			_ => Err(anyhow!("wrong state")),
		}
	}

	pub(super) fn seek(&mut self, position: Duration) -> Result<()> {
		match self {
			Playing(Playing { offset, .. }) => {
				*offset = position;
				Ok(())
			}
			_ => Err(anyhow!("wrong state")),
		}
	}
}
