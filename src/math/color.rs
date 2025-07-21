use std::ops::{Mul, MulAssign};

/// RGBA color
/// Each component is clamped to `0.0..=1.0`
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Color {
	pub red: f32,
	pub green: f32,
	pub blue: f32,
	pub alpha: f32,
}
impl Color {
	pub const WHITE: Self = Self::new(1.0, 1.0, 1.0);
	pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);
	pub const TRANSPARENT: Self = Self::BLACK.alpha(0.0);

	pub const fn new(red: f32, green: f32, blue: f32) -> Self {
		Self {
			red,
			green,
			blue,
			alpha: 1.0,
		}
	}
	pub const fn hex(hex: u32) -> Self {
		let red = ((hex & 0xFF0000) >> 16) as f32 / 255.0;
		let green = ((hex & 0x00FF00) >> 8) as f32 / 255.0;
		let blue = (hex & 0x0000FF) as f32 / 255.0;
		Self::new(red, green, blue)
	}
	pub const fn gray(value: f32) -> Self {
		Self::new(value, value, value)
	}

	pub const fn alpha(mut self, alpha: f32) -> Self {
		self.alpha = alpha;
		self
	}

	pub const fn into_float3(self) -> (f32, f32, f32) {
		(self.red, self.green, self.blue)
	}
	pub const fn into_float4(self) -> (f32, f32, f32, f32) {
		(self.red, self.green, self.blue, self.alpha)
	}
}
impl From<(f32, f32, f32)> for Color {
	fn from(value: (f32, f32, f32)) -> Self {
		Self::new(value.0, value.1, value.2)
	}
}
impl Mul<f32> for Color {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self::Output {
		Self {
			red: self.red * rhs,
			green: self.green * rhs,
			blue: self.blue * rhs,
			alpha: 1.0,
		}
	}
}
impl MulAssign<f32> for Color {
	fn mul_assign(&mut self, rhs: f32) {
		*self = *self * rhs;
	}
}
