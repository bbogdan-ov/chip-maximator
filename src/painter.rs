mod binding;
mod canvas;
mod icon;
mod merge;
mod shader;
mod sprite;
mod text;
mod texture;

pub use canvas::*;
pub use icon::*;
pub use merge::*;
pub use shader::*;
pub use sprite::*;
pub use text::*;
pub use texture::*;

use miniquad::{raw_gl::*, window};

use binding::{Binding, BindingId, Index, VertAttr, Vertex};

use crate::math::{Color, Point, Rect};

#[rustfmt::skip]
pub const QUAD_UV: [Point; 4] = [
	Point::new(0.0, 0.0),
	Point::new(1.0, 0.0),
	Point::new(1.0, 1.0),
	Point::new(0.0, 1.0),
];
#[rustfmt::skip]
pub const QUAD_FLIPPED_UV: [Point; 4] = [
	Point::new(0.0, 1.0),
	Point::new(1.0, 1.0),
	Point::new(1.0, 0.0),
	Point::new(0.0, 0.0),
];

#[derive(Debug, PartialEq)]
pub struct BatchUniforms {
	pub flags: BatchFlag,
	/// Foreground tint color
	pub foreground: Color,
	/// Background color
	pub background: Color,

	pub blend_mode: BlendMode,
	/// Factor of something.
	/// Based on the context of usage.
	pub factor: f32,
	/// Mouse position clamped to 0.0..=1.0
	pub mouse_pos: Point,
}
impl Default for BatchUniforms {
	fn default() -> Self {
		Self {
			flags: BatchFlag::default(),
			foreground: Color::WHITE,
			background: Color::TRANSPARENT,

			blend_mode: BlendMode::Normal,
			factor: 1.0,
			mouse_pos: Point::default(),
		}
	}
}

bitflags::bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub struct BatchFlag: i32 {
		const SPRITE = 1 << 0;
		const TEXT = 1 << 1;
		const MERGE = 1 << 2;

		const PICKER_BG = 1 << 3;
	}
}
impl Default for BatchFlag {
	fn default() -> Self {
		Self::SPRITE
	}
}

/// Rendering context
/// I don't delete resources manually, they will be deleted automaticaly anyway
pub struct PainterContext {
	bindings: Vec<Binding>,
	canvases: Vec<Canvas>,
}
impl Default for PainterContext {
	fn default() -> Self {
		Self {
			bindings: Vec::with_capacity(1),
			canvases: Vec::with_capacity(4),
		}
	}
}

/// Painter
pub struct Painter {
	pub context: PainterContext,

	pub empty_texture: Texture,
	pub white_texture: Texture,

	/// Current batch canvas
	batch_canvas: Option<CanvasId>,
	batch_canvas_changed: bool,
	batch_update_scissor: bool,
	/// Current batch clip rect
	batch_clip: Option<Rect>,
	/// Current batch textures
	batch_textures: Option<(Texture, Texture)>,
	/// Current batch uniforms
	/// Any changes will affect the entire current batch
	batch_uniforms: BatchUniforms,
	/// Number of quads to be drawn in the current batch
	batch_shader: Shader,
	batch_binding: BindingId,
	batch_quads: usize,
	batch_vertices: [Vertex; Self::BATCH_MAX_VERTICES],
	batch_indices: [Index; Self::BATCH_MAX_INDICES],
}
impl Painter {
	pub const BATCH_MAX_QUADS: usize = 200;
	pub const BATCH_MAX_VERTICES: usize = Self::BATCH_MAX_QUADS * 4 * 5; // 5 floats per vertex, 4 vertices per quad
	pub const BATCH_MAX_INDICES: usize = Self::BATCH_MAX_QUADS * 6; // 6 indices per quad

	pub fn new() -> Result<Self, ShaderError> {
		let mut context = PainterContext::default();

		unsafe {
			glEnable(GL_BLEND);

			glBlendEquationSeparate(GL_FUNC_ADD, GL_FUNC_ADD);
			glBlendFuncSeparate(
				GL_SRC_ALPHA,
				GL_ONE_MINUS_SRC_ALPHA,
				GL_ONE,
				GL_ONE_MINUS_SRC_ALPHA,
			);
		}

		let batch_shader = context.new_shader(
			crate::include_shader!("batch.vert.glsl"),
			crate::include_shader!("batch.frag.glsl"),
			&[
				"u_texture1",
				"u_texture2",
				"u_view_size_px",
				"u_flags",
				"u_foreground",
				"u_background",
				"u_blend_mode",
				"u_factor",
				"u_mouse_pos",
			],
		)?;
		let batch_binding = context.new_bindings(
			Self::BATCH_MAX_VERTICES,
			Self::BATCH_MAX_INDICES,
			&[VertAttr::Float2, VertAttr::Float2, VertAttr::Float1],
		);

		Ok(Self {
			empty_texture: context.new_texture(1, 1, Some(&[0, 0, 0, 0]), Default::default()),
			white_texture: context.new_texture(
				1,
				1,
				Some(&[255, 255, 255, 255]),
				Default::default(),
			),

			batch_canvas: None,
			batch_canvas_changed: true,
			batch_update_scissor: false,
			batch_clip: None,
			batch_textures: None,
			batch_uniforms: BatchUniforms::default(),
			batch_shader,
			batch_binding,
			batch_quads: 0,
			batch_vertices: [0.0; Self::BATCH_MAX_VERTICES],
			batch_indices: [0; Self::BATCH_MAX_INDICES],

			context,
		})
	}

	pub fn begin_frame(&mut self) {
		for canvas in self.context.canvases.iter_mut() {
			canvas.damaged = false;
		}
	}
	pub fn commit_frame(&mut self) {
		// Draw current batch at the end of every frame
		self.draw();

		// Clear all not damaged canvases
		for id in 0..self.context.canvases.len() {
			let id = CanvasId(id);
			let canvas = self.canvas(id);
			if !canvas.damaged && canvas.data.clear {
				self.clear(Some(id));
			}
		}
	}

	/// FIXME: this code is mostly the same as in [`Painter::draw`]
	pub fn clear(&mut self, canvas: Option<CanvasId>) {
		let CanvasData { color, size, .. } = self.canvas_data(canvas);

		self.draw();

		self.context.apply_canvas(canvas);

		unsafe {
			glViewport(0, 0, size.x as i32, size.y as i32);
			match self.batch_clip {
				Some(rect) => glScissor(
					rect.pos.x as i32,
					rect.pos.y as i32,
					rect.size.x as i32,
					rect.size.y as i32,
				),
				None => glScissor(0, 0, size.x as i32, size.y as i32),
			}

			glClearColor(color.red, color.green, color.blue, color.alpha);
			glClear(GL_COLOR_BUFFER_BIT);
		}

		self.context.apply_canvas(None);
	}
	/// Draw the current batch and start a new one
	pub fn draw(&mut self) {
		if self.batch_quads == 0 {
			return;
		}

		// Begin render pass of the new canvas
		let CanvasData {
			color,
			size: view_size,
			clear,
		} = self.canvas_data(self.batch_canvas);

		if self.batch_canvas_changed {
			self.context.apply_canvas(self.batch_canvas);

			unsafe {
				glViewport(0, 0, view_size.x as i32, view_size.y as i32);

				if clear {
					glClearColor(color.red, color.green, color.blue, color.alpha);
					glClear(GL_COLOR_BUFFER_BIT);
				}
			}

			self.batch_canvas_changed = false;
		}
		if self.batch_update_scissor {
			unsafe {
				if let Some(rect) = self.batch_clip {
					glEnable(GL_SCISSOR_TEST);

					glScissor(
						rect.pos.x as i32,
						rect.pos.y as i32,
						rect.size.x as i32,
						rect.size.y as i32,
					);
				} else {
					glDisable(GL_SCISSOR_TEST);
				}
			}

			self.batch_update_scissor = false;
		}

		// Update buffers
		self.context.update_binding(
			self.batch_binding,
			&self.batch_vertices[..self.batch_quads * 4 * 5],
			&self.batch_indices[..self.batch_quads * 6],
		);

		// Apply binding
		self.context.apply_binding(Some(self.batch_binding));

		// Apply batch shader
		self.context.apply_shader(Some(&self.batch_shader));

		// Apply uniforms
		let uni = &self.batch_uniforms;
		crate::apply_uniforms! {
			self.context, &self.batch_shader;
			"u_view_size_px" => view_size.into_tuple(),

			"u_flags" => uni.flags.bits(),
			"u_foreground" => uni.foreground.into_float3(),
			"u_background" => uni.background.into_float4(),
			"u_blend_mode" => Uniform::Int1(uni.blend_mode.into()),
			"u_factor" => uni.factor,
			"u_mouse_pos" => uni.mouse_pos.into_tuple(),
		}

		// Apply textures
		if let Some(tex) = self.batch_textures {
			crate::apply_uniforms! {
				self.context, &self.batch_shader;
				"u_texture1" => Uniform::Texture(0, tex.0),
				"u_texture2" => Uniform::Texture(1, tex.1),
			}
		}

		unsafe {
			// Draw triangles
			glDrawElements(
				GL_TRIANGLES,
				self.batch_quads as i32 * 6,
				GL_UNSIGNED_INT,
				std::ptr::null(),
			);
		}

		self.reset();
	}
	/// Reset current batch
	pub fn reset(&mut self) {
		self.batch_quads = 0;
		self.batch_uniforms = BatchUniforms::default();
	}

	/// Set current batch clip rect
	pub fn set_clip(&mut self, rect: Option<Rect>) {
		if self.batch_clip != rect {
			self.draw();
			self.batch_clip = rect;
			self.batch_update_scissor = true;
		}
	}

	/// Set current batch uniforms
	pub fn set_uniforms(
		&mut self,
		canvas: Option<CanvasId>,
		textures: Option<(Texture, Texture)>,
		uniforms: BatchUniforms,
	) {
		let canvas_changed = self.batch_canvas != canvas;

		if canvas_changed || self.batch_textures != textures || self.batch_uniforms != uniforms {
			self.draw();
		}

		if canvas_changed {
			self.batch_canvas_changed = true;
			self.batch_update_scissor = true;
		}

		self.batch_canvas = canvas;
		self.batch_textures = textures;
		self.batch_uniforms = uniforms;
	}

	#[allow(clippy::identity_op)]
	pub fn push_verts(&mut self, verts: [Point; 4], uv: [Point; 4], opacity: f32) {
		let quads = self.batch_quads;
		let vertices = &mut self.batch_vertices;
		let indices = &mut self.batch_indices;

		for i in 0..4 {
			vertices[quads * 4 * 5 + i * 5 + 0] = verts[i].x;
			vertices[quads * 4 * 5 + i * 5 + 1] = verts[i].y;
			vertices[quads * 4 * 5 + i * 5 + 2] = uv[i].x;
			vertices[quads * 4 * 5 + i * 5 + 3] = uv[i].y;
			vertices[quads * 4 * 5 + i * 5 + 4] = opacity;
		}

		indices[quads * 6 + 0] = quads as u32 * 4 + 0;
		indices[quads * 6 + 1] = quads as u32 * 4 + 1;
		indices[quads * 6 + 2] = quads as u32 * 4 + 2;
		indices[quads * 6 + 3] = quads as u32 * 4 + 2;
		indices[quads * 6 + 4] = quads as u32 * 4 + 3;
		indices[quads * 6 + 5] = quads as u32 * 4 + 0;

		self.batch_quads += 1;

		// Damage current batch canvas
		if let Some(id) = self.batch_canvas {
			self.canvas_mut(id).damaged = true;
		}

		if self.batch_quads >= Self::BATCH_MAX_QUADS {
			self.draw();
		}
	}
	/// Push quad into the current batch
	pub fn push_quad(
		&mut self,
		pos: impl Into<Point>,
		size: impl Into<Point>,
		uv: [Point; 4],
		opacity: f32,
	) {
		let pos: Point = pos.into();
		let size: Point = size.into();

		#[rustfmt::skip]
		let verts = [
			Point::new(pos.x,          pos.y),
			Point::new(pos.x + size.x, pos.y),
			Point::new(pos.x + size.x, pos.y + size.y),
			Point::new(pos.x,          pos.y + size.y),
		];

		self.push_verts(verts, uv, opacity);
	}

	/// Returns reference to the [`Canvas`] by its id
	pub fn canvas(&self, id: CanvasId) -> &Canvas {
		&self.context.canvases[id.0]
	}
	/// Returns mut reference to the [`Canvas`] by its id
	pub fn canvas_mut(&mut self, id: CanvasId) -> &mut Canvas {
		&mut self.context.canvases[id.0]
	}
	pub fn canvas_data(&self, id: Option<CanvasId>) -> CanvasData {
		match id {
			Some(id) => self.canvas(id).data,
			None => CanvasData {
				color: Color::BLACK,
				size: window::screen_size().into(),
				clear: true,
			},
		}
	}
}
