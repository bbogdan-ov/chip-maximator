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

use crate::math::{Color, Point};

#[rustfmt::skip]
pub const QUAD_UV: [(f32, f32); 4] = [
	(0.0, 0.0),
	(1.0, 0.0),
	(1.0, 1.0),
	(0.0, 1.0),
];
#[rustfmt::skip]
pub const QUAD_FLIPPED_UV: [(f32, f32); 4] = [
	(0.0, 1.0),
	(1.0, 1.0),
	(1.0, 0.0),
	(0.0, 0.0),
];

#[derive(Debug, PartialEq)]
pub struct BatchUniforms {
	flags: BatchFlag,
	/// Foreground tint color
	foreground: Color,
	/// Background color
	background: Color,

	blend_mode: BlendMode,
	factor: f32,
}
impl Default for BatchUniforms {
	fn default() -> Self {
		Self {
			flags: BatchFlag::default(),
			foreground: Color::WHITE,
			background: Color::TRANSPARENT,

			blend_mode: BlendMode::Normal,
			factor: 1.0,
		}
	}
}

bitflags::bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub struct BatchFlag: i32 {
		const SPRITE = 1 << 0;
		const TEXT = 1 << 1;
		const MERGE = 1 << 2;
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
			glEnable(GL_SCISSOR_TEST);
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
			glScissor(0, 0, size.x as i32, size.y as i32);

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
				glScissor(0, 0, view_size.x as i32, view_size.y as i32);

				if clear {
					glClearColor(color.red, color.green, color.blue, color.alpha);
					glClear(GL_COLOR_BUFFER_BIT);
				}
			}

			self.batch_canvas_changed = false;
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
		}

		self.batch_canvas = canvas;
		self.batch_textures = textures;
		self.batch_uniforms = uniforms;
	}

	/// Push quad into the current batch
	pub fn push_quad(
		&mut self,
		pos: impl Into<Point>,
		size: impl Into<Point>,
		uv: [(f32, f32); 4],
		opacity: f32,
	) {
		let pos: Point = pos.into();
		let size: Point = size.into();

		let quads = self.batch_quads;
		let vertices = &mut self.batch_vertices;
		let indices = &mut self.batch_indices;

		macro_rules! vertices {
			($($idx:expr, $x:expr, $y:expr),*$(,)?) => {
				$(
					vertices[quads * 4 * 5 + $idx * 5 + 0] = $x;
					vertices[quads * 4 * 5 + $idx * 5 + 1] = $y;
					vertices[quads * 4 * 5 + $idx * 5 + 2] = uv[$idx].0;
					vertices[quads * 4 * 5 + $idx * 5 + 3] = uv[$idx].1;
					vertices[quads * 4 * 5 + $idx * 5 + 4] = opacity;
				)*
			};
		}
		macro_rules! indices {
			($i1:expr, $i2:expr, $i3:expr, $i4:expr, $i5:expr, $i6:expr$(,)?) => {
				indices[quads * 6 + 0] = quads as u32 * 4 + $i1;
				indices[quads * 6 + 1] = quads as u32 * 4 + $i2;
				indices[quads * 6 + 2] = quads as u32 * 4 + $i3;
				indices[quads * 6 + 3] = quads as u32 * 4 + $i4;
				indices[quads * 6 + 4] = quads as u32 * 4 + $i5;
				indices[quads * 6 + 5] = quads as u32 * 4 + $i6;
			};
		}

		vertices! {
			0, pos.x,          pos.y,
			1, pos.x + size.x, pos.y,
			2, pos.x + size.x, pos.y + size.y,
			3, pos.x,          pos.y + size.y,
		}
		indices! {
			0, 1, 2,
			2, 3, 0,
		}

		self.batch_quads += 1;

		// Damage current batch canvas
		if let Some(id) = self.batch_canvas {
			self.canvas_mut(id).damaged = true;
		}

		if self.batch_quads >= Self::BATCH_MAX_QUADS {
			self.draw();
		}
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
