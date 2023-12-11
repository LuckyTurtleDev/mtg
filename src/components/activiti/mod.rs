use iced::widget::{column, image, scrollable, Image};

use crate::{App, Element};

pub fn view(app: &App) -> Element {
	let images: Vec<Element> = app
		.search_result
		.iter()
		.map(|card| {
			let url = card.image_uris.get("normal");
			let url = url.map(|url| app.url_cache.get_path(url)).flatten();
			Image::<image::Handle>::new(url.unwrap_or_else(|| "/tmp/ferris.png".into()))
				.into()
		})
		.collect();
	scrollable(column(images)).into()
}
