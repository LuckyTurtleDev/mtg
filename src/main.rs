//Clippy starts hating one_cell::sync::Lazy
#![allow(
	clippy::declare_interior_mutable_const,
	clippy::borrow_interior_mutable_const
)]
#![allow(clippy::expect_fun_call)]

use cache::UrlCacher;
use components::top_bar;
use directories::ProjectDirs;
use iced::{executor, widget::column, Application, Command, Settings, Theme};
use log::info;
use once_cell::sync::Lazy;
use reqwest::{Client, Url};
use scryfall::Card;
use std::{fs::create_dir_all, sync::Arc, time::Instant};

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
	url_cache: UrlCacher,
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
			url_cache: Default::default(),
			em: 16,
			main_activiti: MainActiviti::Search
		}
	}
}

#[derive(Debug, Clone)]
enum Message {
	None,
	UrlCacheDownloaded(Url),
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
		let time = Instant::now();
		info!("update");
		let mut commands = Vec::new();
		match message {
			Message::None => (),
			Message::UrlCacheDownloaded(url) => self.url_cache.callback(url),
			Message::SearchSubmit => commands.push(Command::perform(
				mtg::search(self.search.clone()),
				Message::SearchResult
			)),
			Message::Search(search) => self.search = Arc::new(search),
			Message::SearchResult(cards) => self.search_result = cards,
			Message::MainActiviti(activiti) => self.main_activiti = activiti
		}
		for card in self.search_result.iter() {
			let command = card
				.image_uris
				.get("normal")
				.map(|url| self.url_cache.fetch_if_needed(url))
				.flatten();
			if let Some(command) = command {
				commands.push(command);
			}
		}
		info!("update finish in {}µs", time.elapsed().as_micros());
		Command::batch(commands)
	}

	fn view(&self) -> Element {
		let time = Instant::now();
		info!("draw");
		let activiti_view = match self.main_activiti {
			MainActiviti::Search => components::activiti::view(self),
			_ => "TODO".into()
		};
		let element: Element = column!(top_bar::view(self), activiti_view).into();
		info!("draw finish in {}µs", time.elapsed().as_micros());
		element
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
	create_dir_all(cache::URL_CACHE.as_path())
		.expect(&format!("failed to create {:?} dir", cache::URL_CACHE));
	App::run(Settings::default())
}
