use std::{ops::Deref, sync::Arc};

use iced::widget::{column, scrollable, Image};

use crate::{App, Element, CARD_BACK};

pub fn view(app: &App) -> Element {
	let images: Vec<Element> = app
		.search_result
		.iter()
		.map(|card| {
			let url = card.image_uris.get("normal");
			let handle = url
				.and_then(|url| app.img_cache.get(url, &app.url_cache))
				.and_then(|f| app.img_limiter.limit(f))
				.unwrap_or(CARD_BACK.deref().clone());
			Image::new(handle).into()
		})
		.collect();
	scrollable(column(images)).into()
}

/// wrapper around [`Arc<Vec<u8>>], which impl [`AsRef<[u8]>`]
struct ArcVecU8(Arc<Vec<u8>>);

impl AsRef<[u8]> for ArcVecU8 {
	fn as_ref(&self) -> &[u8] {
		self.0.deref()
	}
}
