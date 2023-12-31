use std::sync::Arc;

use log::info;
use scryfall::{search::Search, Card};

pub async fn search(text: Arc<String>) -> Vec<Card> {
	info!("search for {text:?}");
	let res = text.search().await.unwrap();
	let cards: Vec<Card> = res.into_inner().collect();
	println!("found {} for search {text:?}", cards.len());
	cards
}
