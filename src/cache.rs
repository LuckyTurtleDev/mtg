use crate::{Message, CLIENT, DIRS};
use anyhow::Context;
use iced::Command;
use log::{error, info};
use once_cell::sync::Lazy;
use reqwest::Url;
use std::{
	borrow::Borrow,
	collections::{BTreeMap, HashMap},
	hash::Hash,
	mem::replace,
	path::PathBuf,
	sync::Arc
};
use tokio::fs;

pub const URL_CACHE: Lazy<PathBuf> = Lazy::new(|| DIRS.cache_dir().join("img"));

#[derive(Debug)]
enum CacheState {
	Present,
	Downloading
}

#[derive(Debug, Default)]
///Chache content from a Url to a file
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

#[derive(Debug)]
/// keep data in memory.
/// If full the oldest data will be removed.
/// If a cache miss appears, the data will be loaded to memory again.
pub struct Cacher<K, V>
where
	K: Eq + Hash
{
	/// maps the key to (last_acess_time, value)
	data: HashMap<Arc<K>, (u32, V)>,
	/// maps the last acess time to the key used in the HashMap
	last_acess: BTreeMap<u32, Arc<K>>,
	time: u32,
	max_size: usize
}

impl<K, V> Default for Cacher<K, V>
where
	K: Eq + Hash
{
	fn default() -> Self {
		Cacher {
			data: Default::default(),
			last_acess: Default::default(),
			time: 0,
			max_size: 1000
		}
	}
}

impl<K, V> Cacher<K, V>
where
	K: Eq + Hash
{
	/// get key from cache if present
	fn get<Q>(&self, key: &Q) -> Option<&V>
	where
		Arc<K>: Borrow<Q>,
		Q: Hash + Eq + ?Sized
	{
		self.data.get(key).map(|(_, k)| k)
	}

	/// Load missing data to cache and reseltt acess time.
	/// Should be called before `get()`
	fn need_soon<Q>(&mut self, key: &Q)
	where
		Arc<K>: Borrow<Q>,
		Q: Hash + Eq + ?Sized,
		Q: ToOwned<Owned = K>
	{
		let time = self.data.get_mut(key).map(|(time, _value)| time);
		match time {
			Some(time) => {
				// Reset last acess time
				let value = self.last_acess.remove(time);
				self.last_acess
					.insert(self.time, value.unwrap_or_else(|| Arc::new(key.to_owned())));
				let _ = replace(time, self.time);
				self.time += 1;
			},
			None => {
				// TODO: create command to load missing cache
				todo!()
			}
		}
	}

	fn cache_replacement(&mut self) {
		while self.last_acess.len() > self.max_size {
			// remove oldest element from cache
			let key = self.last_acess.pop_first().map(|(_i, key)| key);
			if let Some(key) = key {
				self.data.remove(&key);
			}
		}
	}
}
