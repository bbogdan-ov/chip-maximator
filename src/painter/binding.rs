use std::mem;

use miniquad::gl::*;

pub type Vertex = f32;
pub type Index = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingId(pub usize);

/// Vertex attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertAttr {
	Float1,
	Float2,
}
impl VertAttr {
	pub fn size(&self) -> i32 {
		const F: i32 = mem::size_of::<f32>() as i32;

		#[allow(clippy::identity_op)]
		match self {
			Self::Float1 => F * 1,
			Self::Float2 => F * 2,
		}
	}
	pub fn components(&self) -> i32 {
		match self {
			Self::Float1 => 1,
			Self::Float2 => 2,
		}
	}
	pub fn into_raw_type(self) -> GLenum {
		match self {
			Self::Float1 => GL_FLOAT,
			Self::Float2 => GL_FLOAT,
		}
	}
}

/// Bindings
pub struct Binding {
	pub vertex_array: GLuint,
	pub vertex_buffer: GLuint,
	pub index_buffer: GLuint,
	pub vertices_size: usize,
	pub indices_size: usize,
}

unsafe fn create_buffer<T>(typ: GLenum, len: usize, usage: GLenum) -> GLuint {
	unsafe {
		let mut buffer: GLuint = 0;
		glGenBuffers(1, &mut buffer);
		if buffer == 0 {
			panic!("failed to create buffer object");
		}
		glBindBuffer(typ, buffer);

		glBufferData(
			typ,
			(mem::size_of::<T>() * len) as GLsizeiptr,
			std::ptr::null(),
			usage,
		);

		buffer
	}
}
unsafe fn update_buffer<T>(typ: GLenum, buffer: GLuint, buffer_size: usize, data: &[T]) {
	unsafe {
		glBindBuffer(typ, buffer);
		glBufferData(typ, buffer_size as _, std::ptr::null(), GL_STREAM_DRAW);
		glBufferSubData(
			typ,
			0,
			mem::size_of_val(data) as _,
			data.as_ptr() as *const _,
		);
		glBindBuffer(typ, 0);
	}
}

impl super::PainterContext {
	pub fn new_bindings(
		&mut self,
		vertices_len: usize,
		indices_len: usize,
		attrs: &[VertAttr],
	) -> BindingId {
		unsafe {
			// Create vertex array
			let mut vertex_array: GLuint = 0;
			glGenVertexArrays(1, &mut vertex_array);
			if vertex_array == 0 {
				panic!("failed to create vertex array object");
			}

			glBindVertexArray(vertex_array);

			// Create buffers
			let vertex_buffer =
				create_buffer::<Vertex>(GL_ARRAY_BUFFER, vertices_len, GL_STATIC_DRAW);
			let index_buffer =
				create_buffer::<Index>(GL_ELEMENT_ARRAY_BUFFER, indices_len, GL_STATIC_DRAW);

			// Enable vertex attributes
			let mut stride = 0;
			for attr in attrs.iter() {
				stride += attr.size();
			}

			let mut offset = 0;
			for (i, attr) in attrs.iter().enumerate() {
				glEnableVertexAttribArray(i as GLuint);
				glVertexAttribPointer(
					i as GLuint,
					attr.components(),
					attr.into_raw_type(),
					0,
					stride,
					offset as *const _,
				);

				offset += attr.size();
			}

			glBindVertexArray(0);
			glBindBuffer(GL_ARRAY_BUFFER, 0);
			glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0);

			self.bindings.push(Binding {
				vertex_array,
				vertex_buffer,
				index_buffer,
				vertices_size: mem::size_of::<Vertex>() * vertices_len,
				indices_size: mem::size_of::<Index>() * indices_len,
			});
			BindingId(self.bindings.len() - 1)
		}
	}

	pub fn apply_binding(&self, binding: Option<BindingId>) {
		if let Some(id) = binding {
			let binding = &self.bindings[id.0];
			unsafe {
				glBindVertexArray(binding.vertex_array);
				glBindBuffer(GL_ARRAY_BUFFER, binding.vertex_buffer);
				glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, binding.index_buffer);
			}
		} else {
			unsafe {
				glBindVertexArray(0);
				glBindBuffer(GL_ARRAY_BUFFER, 0);
				glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0);
			}
		}
	}
	pub fn update_binding(&self, binding: BindingId, vertices: &[Vertex], indices: &[Index]) {
		let binding = &self.bindings[binding.0];

		unsafe {
			update_buffer(
				GL_ARRAY_BUFFER,
				binding.vertex_buffer,
				binding.vertices_size,
				vertices,
			);
			update_buffer(
				GL_ELEMENT_ARRAY_BUFFER,
				binding.index_buffer,
				binding.indices_size,
				indices,
			);
		}
	}
}
