use miniquad::raw_gl::*;

/// RGBA8 texture object ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Texture(pub GLuint);

/// Texture options
#[derive(Debug, Clone, Copy)]
pub struct TextureOpts {
	pub alpha: bool,
	pub min_nearest: bool,
	pub mag_nearest: bool,
}
impl Default for TextureOpts {
	fn default() -> Self {
		Self {
			alpha: true,
			min_nearest: false,
			mag_nearest: true,
		}
	}
}

unsafe fn set_texture_data(width: i32, height: i32, alpha: bool, data: Option<&[u8]>) {
	if let Some(data) = data {
		let channels = if alpha { 4 } else { 3 };
		assert!(
			width * height * channels == data.len() as i32,
			"length of texture data must equal to texture width * height * channels ({channels}) (width = {width}, height = {height}, data.len() = {})",
			data.len()
		);
	}

	unsafe {
		let format = if alpha { GL_RGBA } else { GL_RGB };

		glTexImage2D(
			GL_TEXTURE_2D,
			0,
			format as i32,
			width,
			height,
			0,
			format,
			GL_UNSIGNED_BYTE,
			data.map(|d| d.as_ptr() as _).unwrap_or(std::ptr::null()),
		);
	}
}

impl super::PainterContext {
	pub fn new_texture(
		&mut self,
		width: i32,
		height: i32,
		data: Option<&[u8]>,
		opts: TextureOpts,
	) -> Texture {
		unsafe {
			let mut texture: GLuint = 0;
			glGenTextures(1, &mut texture);
			if texture == 0 {
				panic!("failed to create texture object");
			}

			glBindTexture(GL_TEXTURE_2D, texture);

			let min_filter = if opts.min_nearest {
				GL_NEAREST
			} else {
				GL_LINEAR
			};
			let mag_filter = if opts.mag_nearest {
				GL_NEAREST
			} else {
				GL_LINEAR
			};

			glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, min_filter as i32);
			glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, mag_filter as i32);

			set_texture_data(width, height, opts.alpha, data);

			glBindTexture(GL_TEXTURE_2D, 0);

			Texture(texture)
		}
	}

	pub fn update_texture(
		&self,
		texture: Texture,
		width: i32,
		height: i32,
		alpha: bool,
		data: Option<&[u8]>,
	) {
		unsafe {
			glBindTexture(GL_TEXTURE_2D, texture.0);
			set_texture_data(width, height, alpha, data);
			glBindTexture(GL_TEXTURE_2D, 0);
		}
	}
}
