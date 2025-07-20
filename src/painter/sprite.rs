use miniquad::CursorIcon;

use crate::{
	assets::AssetTexture,
	input::Input,
	math::{Color, Point, Rect},
	util::Anim,
};

use super::{BatchFlag, Canvas, CanvasId, Painter, QUAD_FLIPPED_UV, texture::Texture};

/// Draw sprite descriptor
/// Can be created on each frame draw call
pub struct Sprite {
	pub texture: Texture,
	pub uv_texture: Option<Texture>,
	pub pos: Point,
	pub size: Point,
	/// Number of frames of X and Y axis
	pub frames_count: Point<i32>,
	/// Current frame index on each axis
	pub frame: Point<i32>,
	pub flip: Point<bool>,
	/// Crop sprite, clamped to range `0.0..=1.0`
	/// Sprite will be cropped bottom-to-top, right-to-left
	pub crop: Point,
	pub opacity: f32,
	pub foreground: Color,
	pub background: Color,
}
impl Sprite {
	pub fn new(texture: Texture, size: impl Into<Point>) -> Self {
		let size: Point = size.into();
		Self {
			texture,
			uv_texture: None,
			pos: Point::default(),
			size,
			frames_count: Point::new(1, 1),
			frame: Point::default(),
			flip: Point::default(),
			crop: (1.0, 1.0).into(),
			opacity: 1.0,
			foreground: Color::WHITE,
			background: Color::TRANSPARENT,
		}
	}

	pub fn with_pos(mut self, pos: impl Into<Point>) -> Self {
		self.pos = pos.into();
		self
	}
	pub fn with_size(mut self, size: impl Into<Point>) -> Self {
		self.size = size.into();
		self
	}
	pub fn with_scale(mut self, scale: f32) -> Self {
		self.size = self.size * scale;
		self
	}
	pub fn with_frame(mut self, frame: impl Into<Point<i32>>) -> Self {
		self.frame = frame.into();
		self
	}
	pub fn with_frames_count(mut self, frames: impl Into<Point<i32>>) -> Self {
		self.frames_count = frames.into();
		self
	}
	/// Apply custom UV texture for this sprite
	///
	/// # Note
	///
	/// Note that when custom UV is applied `frame` and `frames_count` will ONLY affect this UV
	/// texture
	pub fn with_uv(mut self, texture: Texture) -> Self {
		self.uv_texture = Some(texture);
		self
	}
	pub fn with_crop(mut self, crop: impl Into<Point>) -> Self {
		self.crop = crop.into();
		self
	}
	pub fn with_opacity(mut self, opacity: f32) -> Self {
		self.opacity = opacity;
		self
	}
	pub fn with_flip(mut self, flip: impl Into<Point<bool>>) -> Self {
		self.flip = flip.into();
		self
	}
	pub fn with_anim(mut self, anim: &Anim) -> Self {
		self.frame.x = anim.frame;
		self
	}
	pub fn with_fg(mut self, color: impl Into<Color>) -> Self {
		self.foreground = color.into();
		self
	}
	pub fn with_bg(mut self, color: impl Into<Color>) -> Self {
		self.background = color.into();
		self
	}

	/// Changes cursor icon to pointer and returns whether the mouse is hovering the sprite
	pub fn is_hover(&self, input: &mut Input) -> bool {
		if !input.is_consumed() && self.rect().contains(&input.mouse_pos()) {
			input.cursor_icon = CursorIcon::Pointer;
			true
		} else {
			false
		}
	}

	fn impl_draw(&self, painter: &mut Painter, canvas: Option<CanvasId>) {
		let frames_x = self.frames_count.x as f32;
		let frames_y = self.frames_count.y as f32;
		let frame_x = self.frame.x as f32;
		let frame_y = self.frame.y as f32;

		let mut uv = QUAD_FLIPPED_UV;
		for row in &mut uv {
			if self.flip.x {
				row.0 = 1.0 - row.0;
			}
			if self.flip.y {
				row.1 = 1.0 - row.1;
			}

			// Doing some calculations to crop the current frame
			row.0 = (row.0 + frame_x) / frames_x;
			row.1 = (row.1 + (frames_y - 1.0 - frame_y)) / frames_y;

			row.0 *= self.crop.x;
			row.1 *= self.crop.y;
		}

		let minus_size = Point::new(
			self.size.x * (1.0 - self.crop.x),
			self.size.y * (1.0 - self.crop.y),
		);

		let mut pos = self.pos;
		let mut size = self.size;

		if self.uv_texture.is_some() {
			// Stretch sprite to the entire screen when using custom UV
			pos = Point::default();
			size = painter.canvas_data(canvas).size;
		}

		pos.y += minus_size.y;

		let uv_texture = self.uv_texture.unwrap_or(painter.empty_texture);
		painter.set_uniforms(
			canvas,
			Some((self.texture, uv_texture)),
			super::BatchUniforms {
				flags: BatchFlag::SPRITE,
				foreground: self.foreground,
				background: self.background,
				..Default::default()
			},
		);
		painter.push_quad(pos, size - minus_size, uv, self.opacity);
	}
	/// Draw the sprite onto canvas
	pub fn draw(&self, painter: &mut Painter, canvas: CanvasId) {
		self.impl_draw(painter, Some(canvas));
	}
	/// Draw the sprite right onto the screen
	pub fn draw_screen(&self, painter: &mut Painter) {
		self.impl_draw(painter, None);
	}

	pub fn rect(&self) -> Rect {
		Rect::new(self.pos, self.size)
	}
}
impl From<&Canvas> for Sprite {
	fn from(value: &Canvas) -> Self {
		Self::new(value.texture, value.data.size)
	}
}
impl From<&AssetTexture> for Sprite {
	fn from(value: &AssetTexture) -> Self {
		Self::new(value.id, value.size).with_frames_count(value.frames)
	}
}
