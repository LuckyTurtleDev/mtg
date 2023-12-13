//Clippy starts hating one_cell::sync::Lazy
#![allow(
	clippy::declare_interior_mutable_const,
	clippy::borrow_interior_mutable_const
)]
#![allow(clippy::expect_fun_call)]

use cache::{UrlCacher, Cacher};
use components::top_bar;
use directories::ProjectDirs;
use iced::{executor, widget::column, Application, Command, Settings, Theme};
use log::info;
use once_cell::sync::Lazy;
use reqwest::{Client, Url};
use scryfall::Card;
use tokio::sync::mpsc::unbounded_channel;
use std::{fs::create_dir_all, sync::Arc, time::Instant};
use tokio::sync::mpsc::UnboundedSender as Sender;
use tokio::sync::mpsc::UnboundedReceiver as Receiver;
use tokio::sync::Mutex;

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

static  RECEIVER: Lazy<Mutex<Option<Receiver<Message>>>> = Lazy::new(||Mutex::new(None));

#[derive(Debug)]
struct App {
	sender: Sender<Message>,
	search: Arc<String>,
	search_result: Vec<Card>,
	/// cache files to storage
	url_cache: UrlCacher,
	/// cache card img to memory
	card_img_cache: Cacher<Url,Vec<u8>>,
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


#[derive(Debug,Clone)]
enum Message {
	None,
	ReceiverMessages(Vec<Message>),
	/// The url cache has finish a Download sucessfull and save it disk
	UrlCacheDownloadReady(Url),
	/// A reqwest to download a file
	UrlCacheDownload(Url),
	Search(String),
	SearchSubmit,
	SearchResult(Vec<Card>),
	MainActiviti(MainActiviti)
}

/// Workarounds: async closures are unstable
/// see issue #62290 <https://github.com/rust-lang/rust/issues/62290> for more information
async fn ac_recv(mut receiver: Receiver<Message>) -> Vec<Message> {
		let mut messages = Vec::new();
		receiver.recv_many(&mut messages, 500).await;
		let mut guard = RECEIVER.lock().await;
		*guard = Some(receiver);
		messages
}

/// If the view function wants to view a image and a cache miss occures,
/// a [`Message`] will be created, to load the image to memmory.
/// Since the view function can not return an Message, the Message will be send over [`tokio::sync::mpsc`].
/// If [`RECEIVER`] is `Some` the next [`update()`] function create a [`Command`], which will listen to mcps in the background
/// and forward send commands and return the [`Receiver`] back to [`RECEIVER`] after it.
/// The [`Receiver`] itself can not be send over [`Message`], because it does not impl [`Copy`].
impl Application for App {
	type Executor = executor::Default;
	type Flags = ();
	type Message = Message;
	type Theme = Theme;

	fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
		let (sender , receiver) = unbounded_channel();
		let app = App {
			sender: sender.clone(),
			search: Default::default(),
			search_result: Default::default(),
			url_cache: UrlCacher::new(sender),
			card_img_cache: Default::default(),
			em: 16,
			main_activiti: MainActiviti::Search
		};
		(
			app,
			Command::perform(ac_recv(receiver), Message::ReceiverMessages)
		)
	}
	fn title(&self) -> String {
		CARGO_PKG_NAME.to_owned()
	}
	

	fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
		let time = Instant::now();
		info!("update");
		let mut commands = Vec::new();
		if let Ok(mut guard) =  RECEIVER.try_lock() {
			if let Some(receiver) = guard.take(){
				let command = Command::perform(ac_recv(receiver), Message::ReceiverMessages);
				commands.push(command);
			}
		}
		match message {
			Message::ReceiverMessages(messages) => {
				info!("process {} Messages", messages.len());
				for message in messages {
					update(self, message,&mut commands);
				}	
			},
			message => update(self, message, &mut commands),
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

fn update(app: &mut App, message: Message, commands: &mut Vec<Command<Message>>) {
		match message {
			Message::ReceiverMessages(_) => panic!(), //this should be catch before
			Message::None => (),
			Message::UrlCacheDownloadReady(url) => app.url_cache.callback(url),
			Message::UrlCacheDownload(url) => if let Some(command) = app.url_cache.fetch_if_needed(&url){
				commands.push(command);
			},
			Message::SearchSubmit => commands.push(Command::perform(
				mtg::search(app.search.clone()),
				Message::SearchResult
			)),
			Message::Search(search) => app.search = Arc::new(search),
			Message::SearchResult(cards) => app.search_result = cards,
			Message::MainActiviti(activiti) => app.main_activiti = activiti
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
