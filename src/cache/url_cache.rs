use crate::{Message, CLIENT, DIRS};
use anyhow::Context;
use iced::Command;
use log::info;
use once_cell::sync::Lazy;
use reqwest::Url;
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use tokio::{fs, sync::mpsc::UnboundedSender as Sender};

pub const URL_CACHE: Lazy<PathBuf> = Lazy::new(|| DIRS.cache_dir().join("img"));

#[derive(Debug)]
enum CacheState {
	Present,
	Downloading
}

#[derive(Debug)]
///Chache content from a Url to a file
pub struct UrlCacher {
	data: FxHashMap<Url, CacheState>,
	sender: Sender<Message>
}

impl UrlCacher {
	pub fn new(sender: Sender<Message>) -> Self {
		Self {
			data: Default::default(),
			sender
		}
	}

	/// get path to file, if it already present
	pub fn get_path(&self, url: &Url) -> Option<PathBuf> {
		match self.data.get(url) {
			None => {
				let res = self.sender.send(Message::UrlCacheDownload(url.to_owned()));
				if let Err(err) = res {
					panic!("{err}");
				}
				None
			},
			Some(CacheState::Downloading) => None,
			Some(CacheState::Present) => Some(url_to_file(url))
		}
	}

	/// download a file and save it to disk.
	/// Should be called when [`Message::UrlCacheDownload`] was send
	pub fn fetch_if_needed(&mut self, url: &Url) -> Option<Command<Message>> {
		if !self.data.contains_key(url) {
			let path = url_to_file(url);
			if path.exists() {
				self.data.insert(url.clone(), CacheState::Present);
				None
			} else {
				let url = url.to_owned();
				self.data.insert(url.clone(), CacheState::Downloading);
				Some(Command::perform(
					dowload_file(url.clone(), path),
					move |res| {
						if res.is_ok() {
							Message::UrlCacheDownloadReady(url)
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

	/// callback after sucessfull download
	/// Should be called when [`Message::UrlCacheDownloadReady`] was send
	pub fn callback(&mut self, url: Url) {
		self.data.insert(url, CacheState::Present);
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

pub fn url_to_file(url: &Url) -> PathBuf {
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
