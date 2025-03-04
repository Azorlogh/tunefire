use core::fmt;
use std::{rc::Rc, sync::Arc, time::Duration};

use anyhow::Result;
use parking_lot::Mutex;
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct TrackInfo {
	pub duration: Duration,
}

#[derive(Clone)]
pub struct TrackSource {
	pub info: TrackInfo,
	pub sample_rate: f64,
	pub signal: Arc<Mutex<dyn Source>>,
}

impl fmt::Debug for TrackSource {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("TrackSource")
			.field("sample_rate", &self.sample_rate)
			.field("info", &self.info)
			.finish()
	}
}

#[derive(Debug)]
pub enum SourceError {
	General(Box<dyn std::error::Error>),
	EndOfStream,
}

pub trait Source: Send {
	fn seek(&mut self, pos: Duration) -> Result<(), SourceError>;

	fn next(&mut self, buf: &mut [[f32; 2]]) -> Result<(), SourceError>;
}

pub trait SourcePlugin: Send {
	fn name(&self) -> &'static str;

	fn handle_url(&self, url: &Url) -> Option<Result<TrackSource>>;
}
