use iced::widget::{image, Image, column, scrollable};

use crate::{App, Element, cache::CardImage};

pub fn view(app: &App) -> Element {
	let images: Vec<Element> = app
		.search_result
		.iter()
		.map(|card| {
			let card = CardImage(card.id);
			Image::<image::Handle>::new(
				app.card_img_cache
					.get_path(&card).unwrap_or_else(|| "/tmp/ferris.png".into())
			).into()}
		)
		.collect();
	scrollable(
	column(images)).into()
}
