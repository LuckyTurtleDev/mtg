use std::{collections::HashMap, fs::create_dir_all, time::Duration, borrow::Cow};

use directories::ProjectDirs;
use iced::{
	executor,
	widget::{column, image, Image, Text},
	Application, Command, Element, Settings, Theme
};
use log::{error, info};
use once_cell::sync::Lazy;
use reqwest::Client;
use scryfall::Card;
use tokio::{fs, time::sleep, io::AsyncRead};
use uuid::Uuid;

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
const DIRS: Lazy<ProjectDirs> = Lazy::new(|| {
	ProjectDirs::from("io.crates", "LuckyTurtleDev", CARGO_PKG_NAME)
		.expect("failed to get project dirs")
});
const CLIENT: Lazy<Client> = Lazy::new(|| reqwest::Client::new());

#[derive(Debug, Default)]
struct App {
	i: u64,
	card_img_cache: HashMap<Uuid, Cache>
}

#[derive(Debug)]
enum Cache {
	Downloding,
	Present
}

#[derive(Debug)]
enum Message {
	Increase,
	DownloadCardImage(Uuid)
}


async fn empty() {

}

impl Application for App {
	type Executor = executor::Default;
	type Flags = ();
	type Message = Message;
	type Theme = Theme;

	fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
		(
			Default::default(),
			//force to call upadet aftert strat
			Command::perform(empty(), |()| Message::Increase)
		)
	}
	fn title(&self) -> String {
		CARGO_PKG_NAME.to_owned()
	}

	fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
		info!("update");
		match message {
			Message::Increase => self.i += 1,
			Message::DownloadCardImage(id) => {
				self.card_img_cache.insert(id, Cache::Present);
			}
		}
		let mut commands = Vec::new();
		self.i += 1;
		let id = Uuid::parse_str("56ebc372-aabd-4174-a943-c7bf59e5028d").unwrap();
		//todo: https://rust-lang.github.io/rust-clippy/master/index.html#/borrow_interior_mutable_const
		self.card_img_cache.entry(id).or_insert_with(|| {
			let patch = DIRS.cache_dir().join(format!("{id}.png"));
			if !patch.exists() {
				let command = Command::perform(dowload_card_image(id), move |f| {
					f.unwrap();
					Message::DownloadCardImage(id)
				});
				commands.push(command);
				Cache::Downloding
			} else {
				Cache::Present
			}
		});
		Command::batch(commands)
	}

	fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
		info!("draw");
		let id =  Uuid::parse_str("56ebc372-aabd-4174-a943-c7bf59e5028d").unwrap();
		let img = if let Some(Cache::Present) = self.card_img_cache.get(&id) {
			DIRS.cache_dir().join(format!("{id}.png"))
		} else {
    			"/tmp/ferris.png".into()
			};
		error!("{img:?}");
		let image = Image::<image::Handle>::new(img);
		column!(image, Text::new(format!("{}", self.i))).into()
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
	Lazy::force(&DIRS);
	create_dir_all(DIRS.cache_dir()).expect("failed to create cache dir");
	App::run(Settings::default())
}

async fn dowload_card_image(scryfall_id: Uuid) -> anyhow::Result<()> {
	info!("download card image {scryfall_id}");
	let card = Card::scryfall_id(scryfall_id).await?;
	let img = card.image_uris.get("png").unwrap();
	let img = CLIENT
		.get(img.as_str())
		.send()
		.await?
		.error_for_status()?
		.bytes()
		.await?;
	fs::write(DIRS.cache_dir().join(format!("{scryfall_id}.png")), img).await?;
	Ok(())
}
