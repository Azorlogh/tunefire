use anyhow::Result;
use url::Url;

use crate::{plugin::TrackInfo, SourcePlugin, TrackSource};

pub struct LocalPlugin;

impl SourcePlugin for LocalPlugin {
	fn name(&self) -> &'static str {
		"Local"
	}

	fn handle_url(&self, url: &Url) -> Option<Result<TrackSource>> {
		if url.scheme() == "file" {
			if let Ok(path) = url.to_file_path() {
				let source =
					crate::util::symphonia::Source::from_file(path).map(|source| TrackSource {
						info: TrackInfo {
							duration: source.duration,
						},
						sample_rate: source.sample_rate,
						signal: Box::new(source),
					});
				Some(source)
			} else {
				None
			}
		} else {
			None
		}
	}
}
