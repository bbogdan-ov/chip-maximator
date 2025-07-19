use crate::{
	app::AppContext,
	emu::Emu,
	math::{Color, Point},
	painter::{CanvasId, Painter, Sprite, Text, Texture, TextureOpts},
	state::State,
	util::Timer,
};

/// Front board game display
pub struct GameDisplay {
	pub canvas: CanvasId,
	buffer: [u8; Self::BUF_SIZE],
	texture: Texture,

	speed_text_timer: Timer,
}
impl GameDisplay {
	const SIZE: Point = Point::new(
		(Emu::SCREEN_WIDTH * 2) as f32,
		(Emu::SCREEN_HEIGHT * 2) as f32,
	);
	const BUF_SIZE: usize = Emu::SCREEN_BUF_SIZE * 3;
	const PROGRESS_WIDTH: usize = 14;

	pub fn new(ctx: &mut AppContext) -> Self {
		let buffer = [0; Self::BUF_SIZE];

		let texture = ctx.painter.context.new_texture(
			Emu::SCREEN_WIDTH as i32,
			Emu::SCREEN_HEIGHT as i32,
			Some(&buffer),
			TextureOpts {
				alpha: false,
				..Default::default()
			},
		);

		let canvas = ctx.painter.context.new_canvas(
			(Self::SIZE.x, Self::SIZE.y),
			Color::BLACK,
			Default::default(),
		);

		Self {
			canvas,
			buffer,
			texture,

			speed_text_timer: Timer::from_millis(600),
		}
	}

	pub fn update(&mut self, ctx: &AppContext, state: &State) {
		self.speed_text_timer.update(&ctx.time);

		// Show speed text when valve is rotating
		if state.valve.is_rotating {
			self.speed_text_timer.start();
		}
	}

	#[allow(clippy::identity_op)]
	fn update_texture(&mut self, painter: &Painter, emu: &Emu) {
		for i in (0..self.buffer.len()).step_by(3) {
			let buf = &mut self.buffer;

			let min = (255.0 * 0.1) as u8;

			if emu.screen[i / 3] {
				buf[i + 0] = (255.0 * 0.7) as u8;
				buf[i + 1] = (255.0 * 0.8) as u8;
				buf[i + 2] = (255.0 * 0.7) as u8;
			} else {
				// Imitate bad display by fading out each pixel on every frame
				let color = (buf[i] as f32 / 1.5) as u8;

				buf[i + 0] = color.max(min);
				buf[i + 1] = color.max(min);
				buf[i + 2] = color.max(min);
			}
		}

		painter.context.update_texture(
			self.texture,
			Emu::SCREEN_WIDTH as i32,
			Emu::SCREEN_HEIGHT as i32,
			false,
			Some(&self.buffer),
		);
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext, state: &mut State) {
		self.update_texture(&ctx.painter, &state.emu);

		// Draw game screen
		Sprite::new(self.texture, Self::SIZE).draw(&mut ctx.painter, self.canvas);

		// Draw speed text
		if !self.speed_text_timer.finished() {
			let progress_text = self.progress_text(state);

			Text::new(&ctx.assets.ibm_font)
				.with_pos((8.0, Self::SIZE.y - 16.0 - 8.0))
				.with_fg((0.7, 0.8, 0.7))
				.with_bg(Color::gray(0.1))
				.draw_line(&mut ctx.painter, self.canvas, b"cpu speed")
				.draw_chars(&mut ctx.painter, self.canvas, &progress_text);
		}
	}

	fn progress_text(&self, state: &State) -> [u8; Self::PROGRESS_WIDTH] {
		const H_PIPE: u8 = 196;
		const V_PIPE: u8 = 197;
		const V_DOUBLE_PIPE: u8 = 215;

		let mut text = [H_PIPE; Self::PROGRESS_WIDTH];

		let speed_f = state.emu.speed / Emu::MAX_SPEED;
		let cur_idx = (speed_f * (text.len() - 1) as f32) as usize;

		for (i, byte) in text.iter_mut().enumerate() {
			if i == cur_idx {
				// Current speed thumb
				*byte = V_PIPE;
			} else if i == 2 {
				// Normal speed thumb
				*byte = V_DOUBLE_PIPE;
			}
		}

		text
	}
}
