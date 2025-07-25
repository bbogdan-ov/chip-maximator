use std::collections::HashSet;

use miniquad::{CursorIcon, KeyCode, MouseButton, window};

use crate::math::Point;

bitflags::bitflags! {
	///
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
	pub struct InputConsume: u8 {
		const VALVE = 1 << 0;
		const BOARD_ANIM = 1 << 1;
	}
}

/// Input manager
pub struct Input {
	/// Mouse position relative to the canvas
	pub real_mouse_pos: Point,
	/// Mouse position relative to the canvas
	/// Affected by current mouse position transform
	pub mouse_pos: Point,
	/// Mouse position relative to the canvas on the last frame
	pub mouse_last_pos: Point,
	pub mouse_movement: Point,
	pub mouse_is_pressed: bool,
	pub mouse_just_pressed: bool,
	pub mouse_just_released: bool,
	pub mouse_button: MouseButton,

	pub key_just_pressed: bool,
	pub keys_pressed: HashSet<KeyCode>,
	pub keys_just_released: HashSet<KeyCode>,

	pub consumed_by: InputConsume,
	/// Cursor icon to apply at the frame end
	pub cursor_icon: CursorIcon,
	prev_cursor_icon: CursorIcon,
}
impl Default for Input {
	fn default() -> Self {
		Self {
			real_mouse_pos: Point::default(),
			mouse_pos: Point::default(),
			mouse_last_pos: Point::default(),
			mouse_movement: Point::default(),
			mouse_is_pressed: false,
			mouse_just_pressed: false,
			mouse_just_released: false,
			mouse_button: MouseButton::Left,

			key_just_pressed: false,
			keys_pressed: HashSet::default(),
			keys_just_released: HashSet::default(),

			consumed_by: InputConsume::default(),
			cursor_icon: CursorIcon::Default,
			prev_cursor_icon: CursorIcon::Default,
		}
	}
}
impl Input {
	pub fn update_after(&mut self) {
		// Update cursor icon
		if self.prev_cursor_icon != self.cursor_icon {
			window::set_mouse_cursor(self.cursor_icon);
			self.prev_cursor_icon = self.cursor_icon;
		}
		self.cursor_icon = CursorIcon::Default;

		self.mouse_movement = self.mouse_pos - self.mouse_last_pos;
		self.mouse_last_pos = self.mouse_pos;

		self.mouse_just_pressed = false;
		self.mouse_just_released = false;
		self.key_just_pressed = false;
		self.keys_just_released.clear();
	}

	/// Returns whether the mouse button is currently down
	pub fn mouse_is_pressed(&self, button: MouseButton) -> bool {
		self.mouse_is_pressed && self.mouse_button == button && !self.is_consumed()
	}
	/// Returns whether the mouse button was pressed for a single frame
	pub fn mouse_just_pressed(&self, button: MouseButton) -> bool {
		self.mouse_just_pressed && self.mouse_button == button && !self.is_consumed()
	}
	pub fn left_is_pressed(&self) -> bool {
		self.mouse_is_pressed(MouseButton::Left)
	}
	pub fn left_just_pressed(&self) -> bool {
		self.mouse_just_pressed(MouseButton::Left)
	}
	pub fn left_just_released(&self) -> bool {
		self.mouse_button == MouseButton::Left && self.mouse_just_released
	}
	pub fn cursor_shaking(&self, threshold: f32) -> bool {
		self.mouse_movement.len_sq() >= threshold.powi(2)
	}

	/// Returns whether the key is currently down
	pub fn key_is_pressed(&self, key: KeyCode) -> bool {
		!self.is_consumed() && self.keys_pressed.contains(&key)
	}
	/// Returns whether the key was pressed for a single frame
	pub fn key_just_pressed(&self, key: KeyCode) -> bool {
		!self.is_consumed() && self.key_just_pressed && self.keys_pressed.contains(&key)
	}
	/// Returns whether the key was released
	pub fn key_just_released(&self, key: KeyCode) -> bool {
		!self.is_consumed() && self.keys_just_released.contains(&key)
	}

	pub fn set_cur_mouse_transform(&mut self, offset: Point, scale: f32) {
		self.mouse_pos = (self.mouse_pos - offset) / scale;
	}
	pub fn reset_cur_mouse_transform(&mut self) {
		self.mouse_pos = self.real_mouse_pos;
	}

	pub fn consume(&mut self, mask: InputConsume, consume: bool) {
		self.consumed_by.set(mask, consume);
	}

	pub fn is_consumed(&self) -> bool {
		!self.consumed_by.is_empty()
	}
}
