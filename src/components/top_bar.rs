use crate::{App, MainActiviti, Message};
use iced::{
	theme,
	widget::{self, button, row, text_input, Space},
	Alignment, Length
};

fn activiti_button<'a>(
	text: &'static str,
	activiti: MainActiviti,
	app: &'a App
) -> widget::Button<'a, Message, iced::Renderer<<App as iced::Application>::Theme>> {
	let bt = button(text);
	if app.main_activiti == activiti {
		bt.style(theme::Button::Primary)
	} else {
		bt.style(theme::Button::Text)
			.on_press(Message::MainActiviti(activiti))
	}
}

pub fn view(
	app: &App
) -> iced::Element<
	<App as iced::Application>::Message,
	iced::Renderer<<App as iced::Application>::Theme>
> {
	let search = text_input("Search", &app.search).on_input(Message::Search);
	let bt_search = activiti_button("Search", MainActiviti::Search, app);
	let bt_stock = activiti_button("Stock", MainActiviti::Stock, app);
	let bt_decks = activiti_button("Decks", MainActiviti::Decks, app);
	let bt_wishs = activiti_button("Wishes", MainActiviti::Wishlist, app);
	row!(
		bt_search,
		bt_stock,
		bt_decks,
		bt_wishs,
		Space::with_width(Length::Fill),
		search,
		Space::with_width(Length::Fill)
	)
	.align_items(Alignment::Center)
	.padding(app.em / 4)
	.spacing(app.em / 2)
	.into()
}
