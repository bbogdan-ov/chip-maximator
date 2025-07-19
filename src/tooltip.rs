use crate::{app::Time, util::Timer};

/// Tooltip state
pub struct Tooltip {
	pub tooltip: [u8; Self::MAX_LEN],
	pub error: [u8; Self::MAX_LEN],

	pub is_tooltip_visible: bool,
	pub error_timer: Timer,
}
impl Default for Tooltip {
	fn default() -> Self {
		Self {
			tooltip: [0; Self::MAX_LEN],
			error: [0; Self::MAX_LEN],
			is_tooltip_visible: false,
			error_timer: Timer::from_millis(2000),
		}
	}
}
impl Tooltip {
	pub const MAX_LEN: usize = 64;

	pub fn update(&mut self, time: &Time) {
		self.error_timer.update(time);
		self.is_tooltip_visible = false;
	}

	fn set_msg(slice: &mut [u8], msg: &[u8]) {
		let len = msg.len().min(Self::MAX_LEN);
		slice[..len].copy_from_slice(&msg[..len]);
		slice[len..].fill(0);
	}

	/// Set tooltip text
	/// Crops `msg` if its length larger than [`Self::MAX_LEN`]
	pub fn set(&mut self, msg: &[u8]) {
		if !self.error_timer.finished() {
			return;
		}

		Self::set_msg(&mut self.tooltip, msg);
		self.is_tooltip_visible = true;
	}
	/// Set error tooltip text
	/// Crops `msg` if its length larger than [`Self::MAX_LEN`]
	pub fn set_error(&mut self, msg: &[u8]) {
		Self::set_msg(&mut self.error, msg);
		self.error_timer.start();
	}
}
