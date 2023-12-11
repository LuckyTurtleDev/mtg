//Clippy starts hating one_cell::sync::Lazy
#![allow(
	clippy::declare_interior_mutable_const,
	clippy::borrow_interior_mutable_const
)]
#![allow(clippy::expect_fun_call)]

use cache::{CardImage, FileCacher};
use components::top_bar;
use directories::ProjectDirs;
use iced::{
	executor,
	widget::{column, image, Image},
	Application, Command, Settings, Theme
};
use log::info;
use once_cell::sync::Lazy;
use reqwest::Client;
use scryfall::Card;
use std::{fs::create_dir_all, sync::Arc};
use uuid::Uuid;

use crate::components::activiti;

mod cache;
mod components;
mod mtg;

type Element<'a> = iced::Element<
	'a,
	<App as iced::Application>::Message,
	iced::Renderer<<App as iced::Application>::Theme>
>;

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
const DIRS: Lazy<ProjectDirs> = Lazy::new(|| {
	ProjectDirs::from("io.crates", "LuckyTurtleDev", CARGO_PKG_NAME)
		.expect("failed to get project dirs")
});
#[allow(clippy::redundant_closure)] // false positive?
const CLIENT: Lazy<Client> = Lazy::new(|| reqwest::Client::new());

#[derive(Debug)]
struct App {
	search: Arc<String>,
	search_result: Vec<Card>,
	card_img_cache: FileCacher<CardImage>,
	//Font size
	em: u16,
	main_activiti: MainActiviti
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MainActiviti {
	Search,
	Stock,
	Decks,
	Wishlist
}

impl Default for App {
	fn default() -> Self {
		Self {
			search: Default::default(),
			search_result: Default::default(),
			card_img_cache: Default::default(),
			em: 16,
			main_activiti: MainActiviti::Search
		}
	}
}

#[derive(Debug, Clone)]
enum Message {
	None,
	CardImgCache(CardImage),
	Search(String),
	SearchSubmit,
	SearchResult(Vec<Card>),
	MainActiviti(MainActiviti)
}

async fn empty() {}

impl Application for App {
	type Executor = executor::Default;
	type Flags = ();
	type Message = Message;
	type Theme = Theme;

	fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
		(
			Default::default(),
			//force to call upadet aftert start
			Command::perform(empty(), |()| Message::None)
		)
	}
	fn title(&self) -> String {
		CARGO_PKG_NAME.to_owned()
	}

	fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
		info!("update");
		let mut commands = Vec::new();
		match message {
			Message::None => (),
			Message::CardImgCache(id) => self.card_img_cache.update(id),
			Message::SearchSubmit => commands.push(Command::perform(
				mtg::search(self.search.clone()),
				Message::SearchResult
			)),
			Message::Search(search) => self.search = Arc::new(search),
			Message::SearchResult(cards) => self.search_result = cards,
			Message::MainActiviti(activiti) => self.main_activiti = activiti
		}
		for card in self.search_result.iter() {
			let command = self.card_img_cache.fetch_if_needed(CardImage(card.id));
			if let Some(command) = command {
				commands.push(command);
			}
		}
		let img_id =
			CardImage(Uuid::parse_str("56ebc372-aabd-4174-a943-c7bf59e5028d").unwrap());
		if let Some(com) = self.card_img_cache.fetch_if_needed(img_id) {
			commands.push(com)
		};
		Command::batch(commands)
	}

	fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
		info!("draw");
		let card_id =
			CardImage(Uuid::parse_str("56ebc372-aabd-4174-a943-c7bf59e5028d").unwrap());
		let img = self
			.card_img_cache
			.get_path(&card_id)
			.unwrap_or_else(|| "/tmp/ferris.png".into());

		let activiti_view = match self.main_activiti {
			MainActiviti::Search => components::activiti::view(self),
			_ => "TODO".into()
		};

		let image = Image::<image::Handle>::new(img);
		column!(top_bar::view(self), activiti_view).into()
	}

	fn theme(&self) -> Self::Theme {
		Theme::Dark
	}
}
fn main() -> iced::Result {
	my_env_logger_style::builder()
		.filter(Some("wgpu_core"), log::LevelFilter::Warn)
		.filter(Some("wgpu_hal"), log::LevelFilter::Warn)
		.filter(Some("iced_wgpu"), log::LevelFilter::Warn)
		.init();
	create_dir_all(cache::CARD_IMAGE_CACHE_DIR.as_path()).expect(&format!(
		"failed to create {:?} dir",
		cache::CARD_IMAGE_CACHE_DIR
	));
	App::run(Settings::default())
}
