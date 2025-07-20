use std::ops::Add;

use miniquad::CursorIcon;

use crate::input::Input;

use super::Point;

/// Rectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect<T = f32> {
	pub pos: Point<T>,
	pub size: Point<T>,
}
impl<T> Rect<T> {
	pub const fn new(pos: Point<T>, size: Point<T>) -> Self {
		Self { pos, size }
	}
	pub const fn new_xywh(x: T, y: T, width: T, height: T) -> Self {
		Self::new(Point::new(x, y), Point::new(width, height))
	}
}
impl<T: Copy + PartialOrd + Add<Output = T>> Rect<T> {
	/// Returns whether the rect overlaps with `point`
	pub fn contains(&self, point: &Point<T>) -> bool {
		point.x >= self.pos.x
			&& point.y >= self.pos.y
			&& point.x <= self.pos.x + self.size.x
			&& point.y <= self.pos.y + self.size.y
	}
}
impl Rect<f32> {
	/// Extend rect by some margin
	pub fn extend(mut self, margin: f32) -> Self {
		self.pos.x -= margin;
		self.pos.y -= margin;
		self.size.x += margin * 2.0;
		self.size.y += margin * 2.0;
		self
	}

	/// Changes cursor icon to pointer and returns whether the mouse is hovering the rect
	pub fn is_hover(&self, input: &mut Input) -> bool {
		if !input.is_consumed() && self.contains(&input.mouse_pos()) {
			input.cursor_icon = CursorIcon::Pointer;
			true
		} else {
			false
		}
	}

	pub fn center(&self) -> Point {
		self.pos + self.size / 2.0
	}
}
