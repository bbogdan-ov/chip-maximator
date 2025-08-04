use core::f32;

use miniquad::CursorIcon;

use crate::{
	assets::AssetTexture,
	input::Input,
	math::{Color, Point, Rect},
	util::Anim,
};

use super::{BatchFlag, Canvas, CanvasId, Painter, QUAD_FLIPPED_UV, texture::Texture};

// FIXME: sprite struct is getting to big, need to do something with this
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
	/// Sprite rotation angle in degrees
	pub angle: f32,
	/// Rotation origin clamped to range `0.0..=1.0` and relative to sprite's top-left corner
	pub origin: Point,
	/// Crop sprite, clamped to range `0.0..=1.0`
	/// Sprite will be cropped bottom-to-top, right-to-left
	pub crop: Point,
	pub opacity: f32,
	pub foreground: Color,
	pub background: Color,
}
#[allow(unused)]
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
			angle: 0.0,
			origin: Point::new(0.5, 0.5),
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
	pub fn with_angle(mut self, angle: f32) -> Self {
		self.angle = angle;
		self
	}
	pub fn with_origin(mut self, origin: impl Into<Point>) -> Self {
		self.origin = origin.into();
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
		if !input.is_consumed() && self.rect().contains(&input.mouse_pos) {
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
				row.x = 1.0 - row.x;
			}
			if self.flip.y {
				row.y = 1.0 - row.y;
			}

			// Doing some calculations to crop the current frame
			row.x = (row.x + frame_x) / frames_x;
			row.y = (row.y + (frames_y - 1.0 - frame_y)) / frames_y;

			row.x *= self.crop.x;
			row.y *= self.crop.y;
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

		let size = size - minus_size;

		#[rustfmt::skip]
		let mut verts = [
			Point::new(pos.x,          pos.y),
			Point::new(pos.x + size.x, pos.y),
			Point::new(pos.x + size.x, pos.y + size.y),
			Point::new(pos.x,          pos.y + size.y),
		];

		if self.angle != 0.0 {
			// Not the most clear code, but hey, it works

			let rads = self.angle.to_radians();
			let cosine = rads.cos();
			let sine = rads.sin();
			let origin_px = Point::new(size.x * self.origin.x, size.y * self.origin.y);
			let origin = pos + origin_px;

			// Rotate one vertex around `origin`
			macro_rules! rotate {
				($idx:expr) => {{
					let vert = &mut verts[$idx];
					let dx = vert.x - origin.x;
					let dy = vert.y - origin.y;
					vert.x = dx * cosine - dy * sine + origin.x;
					vert.y = dx * sine + dy * cosine + origin.y;
				}};
			}

			rotate!(0);
			rotate!(1);
			rotate!(2);
			rotate!(3);
		}

		painter.push_verts(verts, uv, self.opacity);
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
