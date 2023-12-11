use crate::{Message, CLIENT, DIRS};
use anyhow::Context;
use async_trait::async_trait;
use iced::Command;
use log::{error, info};
use once_cell::sync::Lazy;
use scryfall::Card;
use std::{collections::HashMap, hash::Hash, path::PathBuf};
use tokio::fs;
use uuid::Uuid;

pub const CARD_IMAGE_CACHE_DIR: Lazy<PathBuf> =
	Lazy::new(|| DIRS.cache_dir().join("img"));

/// Cache a file, without loding it
#[async_trait]
pub trait FileCacheAbel {
	/// Path were Value Should be stored
	fn cache_path(&self) -> PathBuf;
	async fn fetch(self) -> anyhow::Result<()>;
	fn sucess_message(self) -> Message;
}

#[derive(Debug)]
enum CacheState {
	Present,
	Downloading
}

/// Cache a remote file to filesystem and store the path to the file
#[derive(Debug, Default)]
pub struct FileCacher<K: FileCacheAbel>(HashMap<K, CacheState>);
impl<K> FileCacher<K>
where
	K: FileCacheAbel + Eq + PartialEq + Hash + Clone + Send + 'static
{
	/// fetch a file if it does not already exist local and will not be downloaded already.
	/// Should be call before `get_path` otherwise get_path will always return `None`.
	pub fn fetch_if_needed(&mut self, key: K) -> Option<Command<Message>> {
		let mut command = None;
		self.0.entry(key.clone()).or_insert_with(|| {
			let patch = key.cache_path();
			if !patch.exists() {
				let sucess_message = key.clone().sucess_message();
				let call_back = |res| match res {
					Err(err) => {
						error!("{err:?}");
						Message::None
					},
					Ok(_) => sucess_message
				};
				command = Some(Command::perform(key.fetch(), call_back));
				CacheState::Downloading
			} else {
				CacheState::Present
			}
		});
		command
	}

	/// get the path, to the cached file. (if file is already cached)
	pub fn get_path(&self, key: &K) -> Option<PathBuf> {
		if let Some(CacheState::Present) = self.0.get(key) {
			Some(key.cache_path())
		} else {
			None
		}
	}

	/// Message generted by sucessfull fetch should update the cacher, by calling this function
	pub fn update(&mut self, key: K) {
		self.0.insert(key, CacheState::Present);
	}
}

/// Wraper around scryfall_id of a Card.
/// Impl FileCacheAbel for Card would make me to assume,
/// that the card will be cache not, and not the card image.
#[derive(Debug, Default, Clone, Eq, Hash, PartialEq)]
pub struct CardImage(pub Uuid);

impl From<Card> for CardImage {
	fn from(value: Card) -> Self {
		CardImage(value.id)
	}
}

#[async_trait]
impl FileCacheAbel for CardImage {
	fn cache_path(&self) -> PathBuf {
		CARD_IMAGE_CACHE_DIR.join(format!("{}.png", self.0))
	}
	async fn fetch(self) -> anyhow::Result<()> {
		info!("download card image {}", self.0);
		let card = Card::scryfall_id(self.0)
			.await
			.with_context(|| "failed to fetch card informations")?;
		let img = card.image_uris.get("jpg").unwrap();
		let img = CLIENT
			.get(img.as_str())
			.send()
			.await?
			.error_for_status()?
			.bytes()
			.await?;
		fs::write(self.cache_path(), img).await?;
		Ok(())
	}

	fn sucess_message(self) -> Message {
		Message::CardImgCache(self)
	}
}
