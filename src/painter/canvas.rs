use miniquad::raw_gl::*;

use crate::math::{Color, Point};

use super::{TextureOpts, texture::Texture};

/// Canvas id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasId(pub usize);

/// Canvas data
#[derive(Debug, Clone, Copy)]
pub struct CanvasData {
	/// Background color
	pub color: Color,
	pub size: Point,
	/// Whether to clear canvas
	pub clear: bool,
}

/// Canvas
pub struct Canvas {
	pub framebuffer: GLuint,
	pub texture: Texture,
	/// Indicated whether something was drawn onto the canvas
	pub damaged: bool,
	pub data: CanvasData,
}
impl Canvas {
	pub fn size(&self) -> Point {
		self.data.size
	}
}

impl super::PainterContext {
	fn impl_new_canvas(
		&mut self,
		size: impl Into<Point>,
		color: impl Into<Color>,
		opts: TextureOpts,
		clear: bool,
	) -> CanvasId {
		let size: Point = size.into();

		unsafe {
			// Create texture
			let texture = self.new_texture(size.x as i32, size.y as i32, None, opts);

			// Create framebuffer
			let mut framebuffer: GLuint = 0;
			glGenFramebuffers(1, &mut framebuffer);
			if framebuffer == 0 {
				panic!("failed to create canvas framebuffer object");
			}

			glBindFramebuffer(GL_FRAMEBUFFER, framebuffer);

			// Attach texture to the framebuffer
			glFramebufferTexture2D(
				GL_FRAMEBUFFER,
				GL_COLOR_ATTACHMENT0,
				GL_TEXTURE_2D,
				texture.0,
				0,
			);

			glBindFramebuffer(GL_FRAMEBUFFER, 0);

			self.canvases.push(Canvas {
				framebuffer,
				texture,
				damaged: false,
				data: CanvasData {
					color: color.into(),
					size,
					clear,
				},
			});
			CanvasId(self.canvases.len() - 1)
		}
	}
	pub fn new_canvas(
		&mut self,
		size: impl Into<Point>,
		color: impl Into<Color>,
		opts: TextureOpts,
	) -> CanvasId {
		self.impl_new_canvas(size, color, opts, true)
	}
	pub fn new_canvas_no_clear(
		&mut self,
		size: impl Into<Point>,
		color: impl Into<Color>,
		opts: TextureOpts,
	) -> CanvasId {
		self.impl_new_canvas(size, color, opts, false)
	}

	pub fn apply_canvas(&self, canvas: Option<CanvasId>) {
		if let Some(id) = canvas {
			let fb = self.canvases[id.0].framebuffer;

			unsafe { glBindFramebuffer(GL_FRAMEBUFFER, fb) };
		} else {
			unsafe { glBindFramebuffer(GL_FRAMEBUFFER, 0) };
		}
	}
}
