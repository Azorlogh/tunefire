use std::{fs, str::FromStr};

use anyhow::Result;
use regex::Regex;
use tf_plugin::{SearchPlugin, SearchResult};
use url::Url;

pub struct LocalSearchPlugin;

// TODO rustify overall
impl SearchPlugin for LocalSearchPlugin {
	fn search(&mut self, query: &str) -> Result<Vec<tf_plugin::SearchResult>> {
		let entries = fs::read_dir(query).unwrap();

		let re = Regex::new(r".*(wav|mp3)$").unwrap();

		let mut results: Vec<SearchResult> = Vec::new();
		for entry in entries.filter_map(|e| e.ok()) {
			let path = entry.path();
			let Some(file_name) = path.file_name() else {
				continue;
			};
			let filename = file_name.to_string_lossy();
			let Some(caps) = re.captures(&filename) else {
				continue;
			};

			// TODO read media file metadatas
			let title = caps[0].to_string();
			let artists = vec![String::from_str("Test").unwrap()];
			let file_prefix = "file://";
			let url = file_prefix.to_string() + entry.path().to_str().unwrap();
			// let artwork: Option<ImageBuf> = Option::None;

			let search_result = SearchResult {
				url: Url::from_str(&url).unwrap().into(),
				artists,
				title,
				// artwork,
			};

			results.push(search_result);
		}

		Ok(results)
	}
}
