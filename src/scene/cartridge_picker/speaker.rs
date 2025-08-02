use core::f32;

use miniquad::{KeyCode, MouseButton};

use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	audio::Sound,
	math::{Color, Lerp, Point},
	painter::{CanvasId, Sprite, Text},
	state::State,
	util::Timer,
};

/// Pronounce
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pronounce {
	Wait(usize),
	None,
	Half,
	A,
	AAA,
	OOO,
	III,
	FFF,
}

#[rustfmt::skip]
const LETTERS_PRONOUNCE: [&[Pronounce]; 16] = {
	use Pronounce::*;

	[
		&[III, OOO, OOO, A], // 0
		&[OOO, AAA, III], // 1
		&[OOO, OOO, A], // 2
		&[FFF, A, A], // 3
		&[FFF, OOO, A], // 4
		&[FFF, AAA, A, III], // 5
		&[III, A], // 6
		&[III, A, OOO, A], // 7
		&[A, OOO, III], // 8
		&[AAA, A, III], // 9
		&[A, A, III], // a
		&[A, A], // b
		&[A, A], // c
		&[A, III], // d
		&[III, III], // e
		&[A, A, FFF], // f
	]
};
#[rustfmt::skip]
const SONG: &[Pronounce] = {
	use Pronounce::*;

	&[
		Wait(10),
		Half,
		None,
		Wait(13),
		AAA, FFF, OOO,
		Wait(2),
		AAA,
		Wait(5),
		FFF, OOO, AAA, III, A, AAA,
		Wait(2),
		A, None, FFF, OOO, Wait(1), FFF, III,
		Wait(2),
		FFF, A,
		Wait(5),
		None,
		Wait(3),
		A, FFF, OOO, Wait(1), AAA,
		Wait(2),
		A, Wait(1), III, OOO, Wait(1), AAA,
		Wait(5),
		A, None, Wait(1), OOO, AAA,
		Wait(3),
		A, Wait(1), AAA, OOO,
		Wait(4),
		FFF, A, Wait(1), None,
		// im resonable man
		A, FFF, A, III, Wait(1), FFF, A, Wait(2), None,
		// get of my case
		A, OOO, Wait(1), FFF, A, Wait(1), AAA, Wait(1), None,
		// get of my case
		A, OOO, Wait(1), FFF, A, Wait(1), AAA, Wait(5), None,
		Wait(7),
		// im resonable man
		FFF, A, III, Wait(1), FFF, A, Wait(2), None,
		// get of my case
		A, OOO, Wait(1), FFF, A, Wait(1), AAA, Wait(2), None,
		// get of my case
		A, OOO, Wait(1), FFF, A, Wait(1), AAA, Wait(1), None,
		// get of my case
		A, OOO, Wait(1), FFF, A, Wait(1), AAA, Wait(1), None,
		Wait(1),
		// im resonable man
		FFF, A, III, Wait(1), FFF, A, Wait(2), None,
		Wait(1),
		// get of my case
		A, OOO, Wait(1), FFF, A, Wait(1), AAA, Wait(1), None,
		Wait(1),
		// get of my case
		A, OOO, Wait(1), FFF, A, Wait(1), AAA, Wait(1), None,
		// get of my case
		A, OOO, Wait(1), FFF, A, Wait(1), AAA, Wait(5), A, Wait(7), None
	]
};

/// Speaker
/// Reads and speaks cartridge program bytes out loud
pub struct Speaker {
	speaking: bool,
	cur_nibble: u8,
	/// Current unsigned 4-bit int index in cartridge ROM
	cur_nibble_idx: usize,
	cur_pronounce: usize,
	/// Delay between nibble pronounciation
	nibble_timer: Timer,
	/// Delay between pronounce animations
	pronounce_timer: Timer,

	speaker_frame: i32,

	wait_times: usize,
	last_millis: u64,
	millis_err: u64,
	song_sound: Sound,

	canvas: CanvasId,
	glitch: bool,
	flicker: bool,
	text_time: f32,
	suffering: bool,
	frame_suffering: bool,
}
impl Speaker {
	const POS: Point = Point::new(
		CANVAS_WIDTH / 2.0 - 310.0 / 2.0,
		CANVAS_HEIGHT / 2.0 - 310.0 / 2.0,
	);

	pub fn new(ctx: &mut AppContext) -> Self {
		Self {
			speaking: false,
			cur_nibble: 0,
			cur_nibble_idx: 0,
			cur_pronounce: 0,
			nibble_timer: Timer::from_millis(1000),
			pronounce_timer: Timer::from_millis(200),

			speaker_frame: 0,

			wait_times: 0,
			last_millis: 0,
			millis_err: 0,
			song_sound: {
				let snd = ctx
					.audio
					.new_sound_from_vorbis(ctx.assets.TEMP_sound, false);
				// snd.set_volume(0.0);
				snd
			},

			canvas: ctx
				.painter
				.context
				.new_canvas_no_clear((256.0, 256.0), Default::default()),
			glitch: false,
			flicker: true,
			suffering: false,
			frame_suffering: false,
			text_time: 0.0,
		}
	}

	pub fn update(&mut self, ctx: &mut AppContext, state: &State) {
		self.update_speaking(ctx, state);
	}
	fn update_speaking(&mut self, ctx: &mut AppContext, state: &State) {
		self.nibble_timer.update(&ctx.time);
		self.pronounce_timer.update(&ctx.time);

		if ctx.input.key_just_pressed(KeyCode::Space) {
			self.speaking ^= true;
			if self.speaking {
				self.song_sound.play();
			} else {
				self.song_sound.pause();
			}
		}

		if !self.speaking {
			return;
		}

		if ctx.input.key_just_pressed(KeyCode::G) {
			self.glitch = true;
		}
		if ctx.input.key_just_pressed(KeyCode::H) {
			self.flicker ^= true;
		}
		if ctx.input.key_just_pressed(KeyCode::J) {
			self.suffering ^= true;
		}
		if ctx.input.key_just_pressed(KeyCode::K) {
			self.frame_suffering = true;
		}

		let Sound::Normal { sink } = &self.song_sound else {
			panic!()
		};

		let millis = sink.get_pos().as_millis() as u64;
		let diff = millis - self.last_millis + self.millis_err;
		if diff >= 200 {
			self.last_millis = millis;
			self.millis_err = diff - 200;
			if self.wait_times > 0 {
				self.wait_times -= 1;
				self.pronounce_timer.start();
				return;
			}

			if self.cur_pronounce >= SONG.len() {
				self.speaker_frame = 2;
				return;
			}
			let pronounce = SONG[self.cur_pronounce];

			match pronounce {
				Pronounce::Wait(n) => self.wait_times = n - 1,
				Pronounce::Half => self.speaker_frame = 1,
				Pronounce::None => self.speaker_frame = 2,
				Pronounce::A => self.speaker_frame = 3,
				Pronounce::AAA => self.speaker_frame = 4,
				Pronounce::OOO => self.speaker_frame = 5,
				Pronounce::III => self.speaker_frame = 6,
				Pronounce::FFF => self.speaker_frame = 7,
			}

			self.cur_pronounce += 1;
			self.pronounce_timer.start();
		}
	}

	fn draw_glitch(&mut self, ctx: &mut AppContext) {
		use quad_rand::rand;

		const DS: f32 = 256.0;

		let step_x = 2.0;
		let step_y = 2.0;
		let slices = quad_rand::gen_range(2, 16);

		// Pick a random frame
		let (frame_x, frame_y) = {
			let idx = quad_rand::gen_range(0, slices * slices);
			(idx % slices, idx / slices)
		};

		let mut mov = Point::default();
		if ctx.input.mouse_is_pressed(MouseButton::Right) {
			mov = ctx.input.mouse_pos - Point::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0);
			mov.x /= 20.0;
			mov.y /= 20.0;
		}

		let offset_x = if rand() % 2 == 0 { step_x } else { -step_x } + mov.x;
		let offset_y = if rand() % 2 == 0 { step_y } else { -step_y } + mov.y;

		// Draw last canvas texture, crop an quarter and offset it by random about of pixels
		let cvs = ctx.painter.canvas(self.canvas);
		let sprite = Sprite::from(cvs)
			.with_pos((
				(DS / slices as f32) * frame_x as f32 + offset_x,
				(DS / slices as f32) * frame_y as f32 + offset_y,
			))
			.with_frames_count((slices, slices))
			.with_frame((frame_x, frame_y))
			.with_scale(1.0 / slices as f32);

		sprite.draw(&mut ctx.painter, self.canvas);
	}
	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		if self.glitch {
			for _ in 0..8 {
				self.draw_glitch(ctx);
			}
		}

		let draw = !self.glitch
			|| self.glitch && (ctx.time.elapsed % 2 == 0 && self.flicker)
			|| (self.suffering && ctx.time.elapsed % 2 == 0)
			|| self.frame_suffering;

		if draw {
			// Draw speaker head
			let speaker_size = ctx.assets.speaker.size;
			let mut sprite = Sprite::from(&ctx.assets.speaker)
				.with_pos(((128.0 - speaker_size.x / 2.0).floor(), 46.0));

			if self.suffering || self.frame_suffering {
				sprite.frame.x = quad_rand::gen_range(8_i32, 16);
				self.frame_suffering = false;
			} else {
				sprite.frame.x = self.speaker_frame;
			}

			sprite.draw(&mut ctx.painter, self.canvas);
		}
	}
	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		// Draw window frame
		Sprite::from(&ctx.assets.speaker_window)
			.with_pos(Self::POS)
			.draw(&mut ctx.painter, canvas);

		// Draw window buttons
		self.draw_buttons(ctx, canvas);

		Sprite::from(ctx.painter.canvas(self.canvas))
			.with_pos((Self::POS.x + 27.0, Self::POS.y + 32.0))
			.draw(&mut ctx.painter, canvas);

		if self.glitch && !self.flicker {
			let mut text = Text::new(&ctx.assets.serif_font).with_bg(Color::TRANSPARENT);

			let el = self.text_time / 50.0;
			let s = b"CHIP MAXIMATOR   CHIP MAXIMATOR   ";
			let len = s.len() as f32;
			for (i, ch) in s.iter().enumerate() {
				let x = (-el + i as f32 / len * f32::consts::PI * 2.0).cos() * 200.0;
				let mut z = (-el + i as f32 / len * f32::consts::PI * 2.0).sin();
				z = (z + 1.0) / 2.0;

				let fade = ((self.text_time - (i as f32) * 10.0) / 50.0).clamp(0.0, 1.0);
				let opacity = 1.0_f32.lerp(0.0, z * 1.6 - 0.3) * fade;
				if opacity <= f32::EPSILON {
					continue;
				}

				text.foreground = Color::gray(opacity);
				text.char_offset_px = 0.0;
				text.font_size = (-0.2_f32).lerp(1.2, 1.0 / (z + 1.0));
				text.pos.set(
					x + CANVAS_WIDTH / 2.0 - 8.0,
					z * -32.0 + CANVAS_HEIGHT / 2.0 + 100.0,
				);
				text.draw_chars(&mut ctx.painter, canvas, &[*ch]);
			}

			self.text_time += 1.0;
		}
	}
	fn draw_buttons(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		let mut button = Sprite::from(&ctx.assets.speaker_button);

		let mut draw = |idx: i32, pos: Point| -> bool {
			button.frame.set(idx, 0);
			button.pos = pos;

			let mut clicked = false;
			if button.is_hover(&mut ctx.input) {
				if ctx.input.left_is_pressed() {
					button.frame.y = 1;
				}

				clicked = ctx.input.left_just_pressed();
			}

			button.draw(&mut ctx.painter, canvas);
			clicked
		};

		// Play button
		if draw(0, Self::POS + Point::new(26.0, 10.0)) && !self.speaking {
			self.speaking = true;
		}
		// Stop button
		if draw(1, Self::POS + Point::new(52.0, 10.0)) {
			self.speaking = false;
			if cfg!(debug_assertions) {
				self.cur_nibble_idx = 0;
			}
		}
	}
}
