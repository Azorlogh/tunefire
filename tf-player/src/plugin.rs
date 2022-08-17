use std::time::Duration;

use anyhow::Result;
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct TrackInfo {
	pub duration: Duration,
}

pub struct TrackSource {
	pub sample_rate: f64,
	pub signal: Box<dyn Source>,
	pub info: TrackInfo,
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
