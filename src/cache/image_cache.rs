use super::{url_cache::url_to_file, UrlCacher};
use crate::Message;
use anyhow::Context;
use iced::{widget::image::Handle, Command};
use log::{error, info};
use parking_lot::Mutex;
use reqwest::Url;
use rustc_hash::FxHashMap;
use std::{
	collections::BTreeMap,
	mem::{self, replace},
	ops::{Deref, DerefMut},
	sync::Arc
};
use tokio::{fs, sync::mpsc::UnboundedSender as Sender};

#[derive(Debug)]
enum CacheState {
	Present(Handle),
	Loading
}

#[derive(Debug)]
/// keep data in memory.
/// If full the oldest data will be removed.
/// If a cache miss appears, the data will be loaded to memory again.
pub struct Cacher {
	sender: Sender<Message>,
	inner: Mutex<InnerCache>,
	max_size: usize
}

#[derive(Debug, Default)]
/// keep image data in memory.
/// If full the oldest data will be removed.
/// If a cache miss appears, the data will be loaded to memory again.
///
/// Since [`App::view`] have only readonly acess to the data.
/// The inner data, which must be mutate is inside a Mutex.
/// This means that [`get()`] can not return an reference, becaus the mutex must be closed.
/// So the data, will be copy. Since [`Handle`] use [`Arc`] intern this should be cheap.
pub struct InnerCache {
	/// maps the key to (last_acess_time, value)
	data: FxHashMap<Arc<Url>, (u64, CacheState)>,
	/// maps the last acess time to the key used in the HashMap
	last_acess: BTreeMap<u64, Arc<Url>>,
	time: u64
}

impl Cacher {
	pub fn new(sender: Sender<Message>) -> Self {
		Self {
			sender,
			inner: Default::default(),
			max_size: 1000
		}
	}

	pub fn get(&self, url: &Url, url_cache: &UrlCacher) -> Option<Handle> {
		let mut guard = self.inner.try_lock().unwrap();
		let guard = guard.deref_mut();
		match guard.data.get_mut(url) {
			Some((time, value)) => {
				// Update last acess time
				let key_in = guard.last_acess.remove(time);
				guard.last_acess.insert(
					guard.time,
					key_in.unwrap_or_else(|| Arc::new(url.to_owned()))
				);
				let _ = replace(time, guard.time);
				guard.time += 1;
				match value {
					CacheState::Present(handle) => Some(handle.to_owned()),
					CacheState::Loading => None
				}
			},
			None => {
				if let Some(_path) = url_cache.get_path(url) {
					let url = Arc::new(url.to_owned());
					guard.last_acess.insert(guard.time, url.clone());
					guard
						.data
						.insert(url.clone(), (guard.time, CacheState::Loading));
					let message = Message::LoadImage(url);
					self.sender.send(message).unwrap();
					guard.time += 1;
				}
				None
			}
		}
	}

	/// should be called when [`Message::LoadImage`] was send
	pub fn fetch(&self, url: Arc<Url>) -> Command<Message> {
		info!("load {:?} to memory", url.as_str());
		Command::perform(create_handle(url.clone()), |res| match res {
			Ok(handle) => Message::LoadImageReady((url, handle)),
			Err(err) => {
				error!("{err}");
				Message::None
			}
		})
	}

	/// insert a value, after loading has finish.
	/// Should be called when [`Message::LoadImageReady`] was send
	pub fn callback(&mut self, url: Arc<Url>, handle: Handle) {
		let mut guard = self.inner.try_lock().unwrap();
		if let Some((_time, value)) = guard.data.get_mut(url.deref()) {
			let _ = mem::replace(value, CacheState::Present(handle));
		}
		// if `None` the data was removed from cache, before loading finish
	}

	pub fn cache_replacement(&mut self) {
		let mut gaurd = self.inner.try_lock().unwrap();
		while gaurd.last_acess.len() > self.max_size {
			// remove oldest element from cache
			let key = gaurd.last_acess.pop_first().map(|(_i, key)| key);
			if let Some(key) = key {
				gaurd.data.remove(&key);
			}
		}
	}
}

async fn create_handle(url: Arc<Url>) -> anyhow::Result<Handle> {
	let path = url_to_file(url.deref());
	let data = fs::read(&path)
		.await
		.with_context(|| format!("failed to read file {path:?}"))?;
	Ok(Handle::from_memory(data))
}
