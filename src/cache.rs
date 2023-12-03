use crate::{Message, DIRS};
use async_trait::async_trait;
use iced::Command;
use log::error;
use once_cell::sync::Lazy;
use scryfall::Card;
use std::{collections::HashMap, hash::Hash, path::PathBuf};

pub const CARD_IMAGE_CACHE_DIR: Lazy<PathBuf> =
	Lazy::new(|| DIRS.cache_dir().join("img"));

/// wraper around Card.
/// Impl Cache for Card would make me to assume,
/// that the card is cahed not, the image.
struct CardImage<'a>(&'a Card);

impl<'a> From<&'a Card> for CardImage<'a> {
	fn from(value: &'a Card) -> Self {
		CardImage(value)
	}
}

/// Cache a file, without loding it
#[async_trait]
trait FileCacheAbel {
	/// Path were Value Should be stored
	fn cache_path(&self) -> PathBuf;
	async fn fetch(self) -> anyhow::Result<()>;
	fn sucess_message(&self) -> Message;
}

enum CacheState {
	Present,
	Downloading
}

struct FileCacher<K: FileCacheAbel>(HashMap<K, CacheState>);
impl<K> FileCacher<K>
where
	K: FileCacheAbel + Eq + PartialEq + Hash + Clone
{
	fn fetch_if_needed(&mut self, key: K) -> Option<Command<Message>> {
		let mut command = None;
		self.0.entry(key.clone()).or_insert_with(|| {
			let patch = key.cache_path();
			if !patch.exists() {
				let sucess_message = key.sucess_message();
				let fut = key.fetch();
				let call_back = |res| match res {
					Err(err) => {
						error!("{err:?}");
						Message::None
					},
					Ok(_) => sucess_message
				};
				command = Some(Command::perform(fut, call_back));
				CacheState::Downloading
			} else {
				CacheState::Present
			}
		});
		command
	}

	/// get the path, to the cached file. (if file is already cached)
	fn get_path(&self, key: &K) -> Option<PathBuf> {
		if let Some(CacheState::Present) = self.0.get(key) {
			Some(key.cache_path())
		} else {
			None
		}
	}
}

/*
trait Cache {
	type Key;
	type Value;
	fn cache_path(&self) -> PathBuf;
	fn download(&self, cache: &mut HashMap<<Self as Cache>::Key, <Self as Cache>::Value>) -> ();
}

/// for downloading image Cache
impl Cache for Card {
	type Key = Uuid;
	type Value = String;

	fn cache_path(&self) -> PathBuf {
		CARD_IMAGE_CACHE_DIR.join(format!("{}.png", self.id.as_u128()))
	}
	fn download(&self, cache: &mut HashMap<<Self as Cache>::Key, <Self as Cache>::Value>) -> () {
	let test = cache.entry(self.id).or_insert_with(|| {
			let patch = self.cache_path();
			let ret = if !patch.exists() {
				let command = Command::perform(crate::dowload_card_image(self.id), move |f| {
					f.unwrap();
					crate::Message::DownloadCardImage(id)
				});
				(Cache::Downloding, Some(command))
			} else {
				(Cache::Present, None)
			}
		});
		todo!()
}
}
*/
