use crate::{Message, CLIENT, DIRS};
use anyhow::Context;
use iced::Command;
use log::{error, info};
use once_cell::sync::Lazy;
use reqwest::Url;
use std::{collections::HashMap, path::PathBuf};
use tokio::fs;

pub const URL_CACHE: Lazy<PathBuf> = Lazy::new(|| DIRS.cache_dir().join("img"));

#[derive(Debug)]
enum CacheState {
	Present,
	Downloading
}

#[derive(Debug, Default)]
pub struct UrlCacher(HashMap<Url, CacheState>);

impl UrlCacher {
	pub fn get_path(&self, url: &Url) -> Option<PathBuf> {
		match self.0.get(url) {
			None => {
				error!("cache miss {:?}", url.as_str());
				None
			},
			Some(CacheState::Downloading) => None,
			Some(CacheState::Present) => Some(url_to_file(url))
		}
	}

	pub fn fetch_if_needed(&mut self, url: &Url) -> Option<Command<Message>> {
		if !self.0.contains_key(url) {
			let path = url_to_file(url);
			if path.exists() {
				self.0.insert(url.clone(), CacheState::Present);
				None
			} else {
				let url = url.to_owned();
				self.0.insert(url.clone(), CacheState::Downloading);
				Some(Command::perform(
					dowload_file(url.clone(), path),
					move |res| {
						if res.is_ok() {
							Message::UrlCacheDownloaded(url)
						} else {
							Message::None
						}
					}
				))
			}
		} else {
			None
		}
	}

	pub fn callback(&mut self, url: Url) {
		self.0.insert(url, CacheState::Present);
	}
}

async fn dowload_file(url: Url, path: PathBuf) -> anyhow::Result<()> {
	info!("download {:?}", url.as_str());
	let img = CLIENT
		.get(url)
		.send()
		.await?
		.error_for_status()?
		.bytes()
		.await?;
	fs::write(&path, img)
		.await
		.with_context(|| format!("failed to wirte to {path:?}"))?;
	Ok(())
}

fn url_to_file(url: &Url) -> PathBuf {
	//TODO: add proper decoding
	let extension = PathBuf::from(url.path());
	let extension = extension.extension();
	let path: String = url
		.as_str()
		.chars()
		.map(|char| if char.is_alphanumeric() { char } else { '_' })
		.collect();
	let mut path = URL_CACHE.join(path);
	if let Some(extension) = extension {
		//extension is needed for the iced image widget
		path.set_extension(extension);
	}
	path
}
