use crate::math::Point;

use super::{BatchFlag, CanvasId, Painter, QUAD_UV, texture::Texture};

/// Blend mode
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
	#[default]
	Normal,
	Screen,
	Add,
	Overlay,
}
impl From<BlendMode> for i32 {
	fn from(value: BlendMode) -> i32 {
		match value {
			BlendMode::Normal => 0,
			BlendMode::Screen => 1,
			BlendMode::Add => 2,
			BlendMode::Overlay => 3,
		}
	}
}

/// Merges two textures together using specified blend mode
pub struct Merge {
	pub background: Texture,
	pub foreground: Texture,
	pub blend_mode: BlendMode,
	pub factor: f32,
	pub pos: Point,
}
impl Merge {
	pub fn new(background: Texture, foreground: Texture, blend_mode: BlendMode) -> Self {
		Self {
			background,
			foreground,
			blend_mode,
			factor: 1.0,
			pos: Point::default(),
		}
	}

	pub fn with_factor(mut self, factor: f32) -> Self {
		self.factor = factor;
		self
	}
	pub fn with_pos(mut self, pos: impl Into<Point>) -> Self {
		self.pos = pos.into();
		self
	}

	pub fn draw(&self, painter: &mut Painter, canvas: CanvasId) {
		let size = painter.canvas(canvas).size();

		painter.set_uniforms(
			Some(canvas),
			Some((self.background, self.foreground)),
			super::BatchUniforms {
				flags: BatchFlag::MERGE,
				blend_mode: self.blend_mode,
				factor: self.factor,
				..Default::default()
			},
		);
		painter.push_quad(self.pos, size, QUAD_UV, 1.0);
	}
}
