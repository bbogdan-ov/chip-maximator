use core::f32;
use std::ops::{Add, Div, Mul, Sub};

use super::Lerp;

/// 2D point
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Point<T = f32> {
	pub x: T,
	pub y: T,
}
impl<T> Point<T> {
	pub const fn new(x: T, y: T) -> Self {
		Self { x, y }
	}

	pub fn set(&mut self, x: T, y: T) {
		self.x = x;
		self.y = y;
	}

	pub fn into_tuple(self) -> (T, T) {
		(self.x, self.y)
	}
}
impl Point<f32> {
	pub fn floor(self) -> Self {
		Self {
			x: self.x.floor(),
			y: self.y.floor(),
		}
	}

	/// Calculate squared vector length
	pub fn len_sq(self) -> f32 {
		self.x.powi(2) + self.y.powi(2)
	}
	/// Calculate vector length
	pub fn len(self) -> f32 {
		self.len_sq().sqrt()
	}

	/// Calculate angle (in radians) between two vectors
	pub fn angle(self, rhs: Self) -> f32 {
		let dot = self.x * rhs.x + self.y * rhs.y;
		let det = self.x * rhs.y - self.y * rhs.x;
		let a = det.atan2(dot);
		if a < 0.0 { a + f32::consts::TAU } else { a }
	}
}
impl<T: Lerp> Lerp for Point<T> {
	fn lerp(self, to: Self, alpha: f32) -> Self {
		Self {
			x: self.x.lerp(to.x, alpha),
			y: self.y.lerp(to.y, alpha),
		}
	}
}
impl<T> From<(T, T)> for Point<T> {
	fn from(value: (T, T)) -> Self {
		Self::new(value.0, value.1)
	}
}

impl<T: Add<Output = T>> Add for Point<T> {
	type Output = Point<T>;

	fn add(self, rhs: Point<T>) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}
impl<T: Sub<Output = T>> Sub for Point<T> {
	type Output = Point<T>;

	fn sub(self, rhs: Point<T>) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}
impl<T: Copy + Mul<Output = T>> Mul<T> for Point<T> {
	type Output = Point<T>;

	fn mul(self, rhs: T) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
		}
	}
}
impl<T: Copy + Div<Output = T>> Div<T> for Point<T> {
	type Output = Point<T>;

	fn div(self, rhs: T) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
		}
	}
}
