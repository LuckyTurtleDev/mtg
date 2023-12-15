use iced::widget::image::Handle;
use nohash_hasher::BuildNoHashHasher;
use parking_lot::Mutex;
use std::{
	collections::HashSet,
	mem::{replace, take},
	ops::DerefMut
};
use tokio::sync::mpsc::UnboundedSender as Sender;

use crate::Message;

#[derive(Debug)]
pub struct ImgLimiter {
	inner: Mutex<Inner>,
	max_new: u64,
	sender: Sender<Message>
}

#[derive(Debug, Default)]
struct Inner {
	//image id is already an hash. So there is no need to map an u64 to another u64
	current_view: HashSet<u64, BuildNoHashHasher<u64>>,
	preview_view: HashSet<u64, BuildNoHashHasher<u64>>,
	new_count: u64,
	limited: bool
}

impl ImgLimiter {
	pub fn new(max_new: u64, sender: Sender<Message>) -> Self {
		Self {
			inner: Default::default(),
			sender,
			max_new
		}
	}

	pub fn limit(&self, handle: Handle) -> Option<Handle> {
		let mut guard = self.inner.try_lock().unwrap();
		let guard = guard.deref_mut();
		if guard.preview_view.get(&handle.id()).is_some() {
			guard.current_view.insert(handle.id());
			return Some(handle);
		}
		if guard.current_view.contains(&handle.id()) {
			return Some(handle);
		}
		if guard.new_count < self.max_new {
			guard.current_view.insert(handle.id());
			guard.new_count += 1;
			return Some(handle);
		}
		guard.limited = true;
		None
	}
	pub fn view_finish(&self) {
		let mut guard = self.inner.try_lock().unwrap();
		let guard = guard.deref_mut();
		let _ = replace(&mut guard.preview_view, take(&mut guard.current_view));
		guard.new_count = 0;
		if guard.limited {
			// request update of view, to draw the missing images
			self.sender.send(Message::None).unwrap();
		}
		guard.limited = false;
	}
}
