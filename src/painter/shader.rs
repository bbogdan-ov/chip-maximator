use std::{collections::HashMap, ffi::CString, fmt::Display};

use miniquad::raw_gl::*;

use super::Texture;

#[macro_export]
macro_rules! include_shader {
	($filename:expr) => {
		include_str!(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/assets/shaders/",
			$filename
		))
	};
}

#[macro_export]
macro_rules! apply_uniforms {
	($context:expr, $shader:expr; $($name:expr => $value:expr),*$(,)?) => {
		$($context.apply_uniform($shader, $name, $value);)*
	};
}

/// Shader error
#[derive(Debug)]
pub enum ShaderError {
	CompileVert(String),
	CompileFrag(String),

	Link(String),
}
impl std::error::Error for ShaderError {}
impl Display for ShaderError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::CompileVert(s) => write!(f, "failed to compile vertex shader: {s}"),
			Self::CompileFrag(s) => write!(f, "failed to compile fragment shader: {s}"),
			Self::Link(s) => write!(f, "failed to link shader program: {s}"),
		}
	}
}

/// Shader uniform value
#[derive(Debug, Clone, Copy)]
pub enum Uniform {
	Texture(i32, Texture),
	Float1(f32),
	Float2((f32, f32)),
	Float3((f32, f32, f32)),
	Float4((f32, f32, f32, f32)),
	Int1(i32),
	Int2((i32, i32)),
}
impl From<f32> for Uniform {
	fn from(value: f32) -> Self {
		Self::Float1(value)
	}
}
impl From<(f32, f32)> for Uniform {
	fn from(value: (f32, f32)) -> Self {
		Self::Float2(value)
	}
}
impl From<(f32, f32, f32)> for Uniform {
	fn from(value: (f32, f32, f32)) -> Self {
		Self::Float3(value)
	}
}
impl From<(f32, f32, f32, f32)> for Uniform {
	fn from(value: (f32, f32, f32, f32)) -> Self {
		Self::Float4(value)
	}
}
impl From<i32> for Uniform {
	fn from(value: i32) -> Self {
		Self::Int1(value)
	}
}
impl From<(i32, i32)> for Uniform {
	fn from(value: (i32, i32)) -> Self {
		Self::Int2(value)
	}
}

/// Shader
pub struct Shader {
	pub program: GLuint,
	/// Uniforms locations table
	pub uniforms: HashMap<String, GLint>,
}

unsafe fn compile_shader(shader: GLuint, source: &str) -> Result<(), String> {
	unsafe {
		let cstring = CString::new(source).unwrap();
		let src = [cstring];
		glShaderSource(shader, 1, src.as_ptr() as *const _, std::ptr::null());
		glCompileShader(shader);

		// Check compile status
		let mut status = 0;
		glGetShaderiv(shader, GL_COMPILE_STATUS, &mut status);
		if status != 1 {
			let mut info_len: i32 = 0;
			glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut info_len);
			if info_len <= 0 {
				panic!("shader compile status != 1, but info log length is 0")
			}

			let mut info = String::with_capacity(info_len as usize);
			info.extend(std::iter::repeat_n('\0', info_len as usize));
			glGetShaderInfoLog(shader, info_len, &mut info_len, info.as_ptr() as *mut _);

			glDeleteShader(shader);

			return Err(info);
		}

		Ok(())
	}
}
unsafe fn check_link_status(program: GLuint) -> Result<(), String> {
	unsafe {
		let mut link_status = 0;
		glGetProgramiv(program, GL_LINK_STATUS, &mut link_status);

		if link_status != 1 {
			let mut info_len = 0;
			glGetProgramiv(program, GL_INFO_LOG_LENGTH, &mut info_len);
			if info_len == 0 {
				return Ok(());
			}

			let info = String::with_capacity(info_len as usize);
			glGetProgramInfoLog(
				program,
				info_len,
				&mut info_len,
				info.as_ptr() as *mut GLchar,
			);
		}

		Ok(())
	}
}

impl super::PainterContext {
	pub fn new_shader(
		&mut self,
		vertex_source: &str,
		fragment_source: &str,
		uniforms: &[&str],
	) -> Result<Shader, ShaderError> {
		unsafe {
			let program = glCreateProgram();
			if program == 0 {
				panic!("failed to create program object");
			}

			let vert_shader: GLuint = glCreateShader(GL_VERTEX_SHADER);
			if vert_shader == 0 {
				panic!("failed to create vertex shader object")
			}
			compile_shader(vert_shader, vertex_source).map_err(ShaderError::CompileVert)?;

			let frag_shader: GLuint = glCreateShader(GL_FRAGMENT_SHADER);
			if frag_shader == 0 {
				panic!("failed to create fragment shader object")
			}
			compile_shader(frag_shader, fragment_source).map_err(ShaderError::CompileFrag)?;

			glAttachShader(program, vert_shader);
			glAttachShader(program, frag_shader);

			glLinkProgram(program);
			if let Err(e) = check_link_status(program) {
				glDeleteShader(vert_shader);
				glDeleteShader(frag_shader);
				glDeleteProgram(program);
				return Err(ShaderError::Link(e));
			}

			glDetachShader(program, vert_shader);
			glDetachShader(program, frag_shader);
			glDeleteShader(vert_shader);
			glDeleteShader(frag_shader);

			let mut uniforms_table = HashMap::default();
			for name in uniforms {
				let n = CString::new(*name).unwrap();
				let loc = glGetUniformLocation(program, n.as_ptr() as _);
				if loc >= 0 {
					uniforms_table.insert(name.to_string(), loc);
				}
			}

			Ok(Shader {
				program,
				uniforms: uniforms_table,
			})
		}
	}

	pub fn apply_shader(&self, shader: Option<&Shader>) {
		unsafe {
			glUseProgram(shader.map(|s| s.program).unwrap_or(0));
		}
	}
	pub fn apply_uniform(&self, shader: &Shader, name: &str, value: impl Into<Uniform>) {
		let Some(loc) = shader.uniforms.get(name) else {
			return;
		};
		let value: Uniform = value.into();

		unsafe {
			match value {
				Uniform::Texture(slot, tex) => {
					glActiveTexture(GL_TEXTURE0 + slot as GLuint);
					glBindTexture(GL_TEXTURE_2D, tex.0);
					glUniform1i(*loc, slot);
				}
				Uniform::Float1(x) => glUniform1f(*loc, x),
				Uniform::Float2((x, y)) => glUniform2f(*loc, x, y),
				Uniform::Float3((x, y, z)) => glUniform3f(*loc, x, y, z),
				Uniform::Float4((x, y, z, w)) => glUniform4f(*loc, x, y, z, w),
				Uniform::Int1(x) => glUniform1i(*loc, x),
				Uniform::Int2((x, y)) => glUniform2i(*loc, x, y),
			}
		}
	}
}
