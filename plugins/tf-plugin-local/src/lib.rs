use anyhow::Result;
use tf_plugin::{
	player::{SourcePlugin, TrackInfo, TrackSource},
	Plugin,
};
use url::Url;

pub struct Local;

impl Plugin for Local {
	fn get_source_plugin(&self) -> Option<Box<dyn tf_plugin::SourcePlugin>> {
		Some(Box::new(LocalSourcePlugin))
	}
}

pub struct LocalSourcePlugin;

impl SourcePlugin for LocalSourcePlugin {
	fn name(&self) -> &'static str {
		"Local"
	}

	fn handle_url(&self, url: &Url) -> Option<Result<TrackSource>> {
		if url.scheme() == "file" {
			if let Ok(path) = url.to_file_path() {
				let source =
					tf_plugin::player::util::symphonia::Source::from_file(path).map(|source| {
						TrackSource {
							info: TrackInfo {
								duration: source.duration,
							},
							sample_rate: source.sample_rate,
							signal: Box::new(source),
						}
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
