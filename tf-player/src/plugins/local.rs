use anyhow::Result;
use url::Url;

use crate::{plugin::SongInfo, SongSource, SourcePlugin};

pub struct LocalPlugin;

impl SourcePlugin for LocalPlugin {
	fn name(&self) -> &'static str {
		"Local"
	}

	fn handle_url(&self, url: &Url) -> Option<Result<SongSource>> {
		if url.scheme() == "file" {
			if let Ok(path) = url.to_file_path() {
				let source =
					crate::util::symphonia::Source::from_file(path).map(|source| SongSource {
						info: SongInfo {
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
