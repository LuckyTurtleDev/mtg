use std::time::Duration;

use iced::{
	executor,
	widget::{column, image, Image, Text},
	Application, Command, Element, Settings, Theme
};
use log::info;
use tokio::time::sleep;

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

struct App {
	i: u64
}

#[derive(Debug)]
enum Message {
	Increase
}

impl Application for App {
	type Executor = executor::Default;
	type Flags = ();
	type Message = Message;
	type Theme = Theme;

	fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
		(
			App { i: 0 },
			Command::perform(sleep(Duration::from_secs(3)), |()| Message::Increase)
		)
	}
	fn title(&self) -> String {
		CARGO_PKG_NAME.to_owned()
	}

	fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
		info!("update");
		self.i += 1;
		Command::perform(sleep(Duration::from_secs(3)), |()| Message::Increase)
	}

	fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
		let image = Image::<image::Handle>::new("/tmp/ferris.png");
		info!("draw");
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
	App::run(Settings::default())
}
