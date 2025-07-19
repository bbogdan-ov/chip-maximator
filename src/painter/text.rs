use crate::math::{Color, Point};

use super::{BatchFlag, CanvasId, Painter, QUAD_FLIPPED_UV, texture::Texture};

pub const MAX_CHARS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CharWidth(pub f32);
impl Default for CharWidth {
	fn default() -> Self {
		Self(1.0)
	}
}

// Allow because `FontLookup` is used only once per font so it is more optimal to store it in
// the stack rather than in the heap (but i might be wrong)
#[allow(clippy::large_enum_variant)]
/// Font chars lookup table
#[derive(Debug)]
pub enum FontLookup {
	/// All 256 ascii chars
	Ascii,
	Custom([u8; MAX_CHARS], [CharWidth; MAX_CHARS]),
}

/// Text font texture
#[derive(Debug)]
pub struct Font {
	pub texture: Texture,
	/// Size of each char in the texture
	pub size: Point,
	/// Number of chars in the font
	pub count: i32,
	pub lookup: FontLookup,
}

/// Draw text descriptor
#[derive(Debug, Clone)]
pub struct Text<'a> {
	pub font: &'a Font,

	pub pos: Point,
	/// Scale of each text char
	pub font_size: f32,
	pub foreground: Color,
	pub background: Color,

	/// First char offset in pixels
	char_offset_px: f32,
	/// Current line offset in chars
	line_offset: f32,
}
impl<'a> Text<'a> {
	pub fn new(font: &'a Font) -> Self {
		Self {
			font,

			pos: Point::default(),
			font_size: 1.0,
			foreground: Color::WHITE,
			background: Color::BLACK,

			char_offset_px: 0.0,
			line_offset: 0.0,
		}
	}

	pub fn with_pos(mut self, pos: impl Into<Point>) -> Self {
		self.pos = pos.into();
		self
	}
	pub fn with_font_size(mut self, size: f32) -> Self {
		self.font_size = size;
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

	fn begin_draw(&mut self, painter: &mut Painter, canvas: CanvasId) {
		painter.set_uniforms(
			Some(canvas),
			Some((self.font.texture, painter.empty_texture)),
			super::BatchUniforms {
				flags: BatchFlag::TEXT,
				foreground: self.foreground,
				background: self.background,
				..Default::default()
			},
		);
	}

	#[inline]
	fn draw_char(&mut self, painter: &mut Painter, byte: u8, offset: Point) {
		if byte == 0 {
			return;
		}

		let size = self.char_size();
		let mut kerning = size.x;
		let findex: f32;

		match self.font.lookup {
			FontLookup::Ascii => {
				findex = byte as f32;
			}
			FontLookup::Custom(table, widths) => {
				findex = table[byte as usize] as f32;
				kerning *= widths[byte as usize].0;
			}
		};

		let f = kerning / size.x;

		let mut uv = QUAD_FLIPPED_UV;
		for row in &mut uv {
			// Doing some calculations to crop the current char
			row.0 = (row.0 * f + findex + (1.0 - f) / 2.0) / self.font.count as f32;
		}
		let pos = Point::new(
			self.pos.x + self.char_offset_px,
			self.pos.y + self.line_offset * size.y,
		) + offset;
		painter.push_quad(pos, (kerning, size.y), uv, 1.0);

		self.char_offset_px += kerning;
	}
	pub fn draw_chars_with(
		&mut self,
		painter: &mut Painter,
		canvas: CanvasId,
		bytes: &[u8],
		transform: impl Fn(usize, u8) -> (u8, Point),
	) -> &mut Self {
		self.begin_draw(painter, canvas);

		for (i, byte) in bytes.iter().enumerate() {
			let (byte, offset) = transform(i, *byte);
			self.draw_char(painter, byte, offset);
		}

		self
	}
	pub fn draw_chars(
		&mut self,
		painter: &mut Painter,
		canvas: CanvasId,
		bytes: &[u8],
	) -> &mut Self {
		self.draw_chars_with(painter, canvas, bytes, |_, b| (b, Point::default()))
	}
	pub fn draw_line(
		&mut self,
		painter: &mut Painter,
		canvas: CanvasId,
		bytes: &[u8],
	) -> &mut Self {
		self.draw_chars(painter, canvas, bytes).new_line()
	}
	pub fn new_line(&mut self) -> &mut Self {
		self.line_offset += 1.0;
		self.char_offset_px = 0.0;
		self
	}
	pub fn draw_str(&mut self, painter: &mut Painter, canvas: CanvasId, s: &str) -> &mut Self {
		self.begin_draw(painter, canvas);

		for byte in s.as_bytes() {
			if *byte == b'\n' {
				self.new_line();
			} else {
				self.draw_char(painter, *byte, Point::default());
			}
		}

		self
	}

	pub fn char_size(&self) -> Point {
		self.font.size * self.font_size
	}
}
