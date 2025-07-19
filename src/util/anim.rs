use std::{cell::RefCell, ops::Range, rc::Rc};

use crate::{app::Time, util::Timer};

pub type AnimRef = Rc<RefCell<Anim>>;

/// Sprite frame-by-frame animation
#[derive(Debug)]
pub struct Anim {
	/// Current frame index
	pub frame: i32,
	pub range: Range<i32>,

	pub playing: bool,
	/// Whether the current playback is reversed
	pub reversed: bool,
	pub looped: bool,

	frame_timer: Timer,
}
impl Anim {
	pub fn new(fps: u64, range: Range<i32>) -> Self {
		Anim {
			frame: range.start,
			range,

			playing: false,
			reversed: false,
			looped: false,

			frame_timer: Timer::from_millis(1000 / fps),
		}
	}

	pub fn with_looped(mut self) -> Self {
		self.looped = true;
		self
	}
	pub fn with_playing(mut self) -> Self {
		self.play(false);
		self
	}

	pub fn play(&mut self, reversed: bool) {
		if self.playing {
			return;
		}

		self.playing = true;
		self.reversed = reversed;

		self.frame_timer.start();
		self.frame_to_start();
	}
	pub fn stop(&mut self) {
		self.playing = false;
	}

	fn frame_to_start(&mut self) {
		if self.reversed {
			self.frame = self.range.end - 1;
		} else {
			self.frame = self.range.start;
		}
	}

	pub fn update(&mut self, time: &Time) {
		if !self.playing {
			return;
		}

		self.frame_timer.update(time);
		if self.frame_timer.finished() {
			// Increment frame
			if self.reversed {
				self.frame -= 1;
			} else {
				self.frame += 1;
			}

			let next = self.wrap_frame();
			if next {
				self.frame_timer.start();
			}
		}
	}

	fn wrap_frame(&mut self) -> bool {
		if self.range.contains(&self.frame) {
			return true;
		}

		if self.looped {
			self.frame_to_start();
			true
		} else {
			self.stop();
			false
		}
	}

	pub fn into_ref(self) -> AnimRef {
		Rc::new(RefCell::new(self))
	}

	pub fn progress(&self) -> f32 {
		if !self.playing {
			return 0.0;
		}

		let (from, to) = (self.range.start, self.range.end);
		// let (from, to) = if self.reversed {
		// 	(self.range.end, self.range.start)
		// } else {
		// 	(self.range.start, self.range.end)
		// };
		(self.frame - from) as f32 / (to - from) as f32
	}
}
impl From<Anim> for AnimRef {
	fn from(value: Anim) -> Self {
		value.into_ref()
	}
}
