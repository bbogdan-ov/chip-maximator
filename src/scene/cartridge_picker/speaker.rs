use crate::{
	app::{AppContext, CANVAS_WIDTH},
	games::GAMES,
	math::{Color, Point, Rect},
	painter::{CanvasId, Sprite, Text},
	state::State,
	util::Timer,
};

use super::PickerState;

/// Pronounce
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pronounce {
	A,
	AAA,
	OOO,
	III,
	FFF,
}

#[rustfmt::skip]
const LETTERS_PRONOUNCE: [&[Pronounce]; 16] = {
	use Pronounce::*;

	// TODO: lips animation looks wrong for some letters/numbers
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

/// Speaker
/// Reads and speaks cartridge program bytes out loud
pub struct Speaker {
	speaking: bool,
	cur_nibble: u8,
	/// Current unsigned 4-bit int index in cartridge ROM
	cur_nibble_idx: usize,
	cur_pronounce: usize,
	is_pronouncing: bool,

	/// Delay between nibble pronounciation
	nibble_timer: Timer,
	/// Delay between pronounce animations
	pronounce_timer: Timer,
	/// Delay before closing eyes
	rest_timer: Timer,

	speaker_frame: i32,
}
impl Speaker {
	const POS: Point = Point::new(CANVAS_WIDTH - 320.0, 30.0);

	pub fn new() -> Self {
		Self {
			speaking: false,
			cur_nibble: 0,
			cur_nibble_idx: 0,
			cur_pronounce: 0,
			is_pronouncing: false,

			nibble_timer: Timer::from_millis(1000),
			pronounce_timer: Timer::from_millis(100),
			rest_timer: Timer::from_millis(1500),

			speaker_frame: 0,
		}
	}

	pub fn update(&mut self, ctx: &mut AppContext, state: &State) {
		self.rest_timer.update(&ctx.time);
		self.nibble_timer.update(&ctx.time);
		self.pronounce_timer.update(&ctx.time);

		self.update_speaking(ctx, state);

		let p = self.rest_timer.progress();
		if p >= 1.0 {
			// Fully close eyes
			self.speaker_frame = 0;
		} else if p > 0.9 {
			// Close eyes in half
			self.speaker_frame = 1;
		}

		if self.pronounce_timer.finished() && self.is_pronouncing {
			let pronounces = LETTERS_PRONOUNCE[self.cur_nibble as usize];
			if self.cur_pronounce >= pronounces.len() {
				self.speaker_frame = 2;
				self.is_pronouncing = false;
				return;
			}

			match pronounces[self.cur_pronounce] {
				Pronounce::A => self.speaker_frame = 3,
				Pronounce::AAA => self.speaker_frame = 4,
				Pronounce::OOO => self.speaker_frame = 5,
				Pronounce::III => self.speaker_frame = 6,
				Pronounce::FFF => self.speaker_frame = 7,
			}

			if self.speaking || self.cur_pronounce > 0 {
				self.cur_pronounce += 1;
				self.pronounce_timer.start();
				self.rest_timer.start();
			}
		}
	}
	fn update_speaking(&mut self, ctx: &mut AppContext, state: &State) {
		if !self.speaking {
			return;
		}

		if self.nibble_timer.finished() {
			let sounds = &[
				ctx.assets.sound_0,
				ctx.assets.sound_1,
				ctx.assets.sound_2,
				ctx.assets.sound_3,
				ctx.assets.sound_4,
				ctx.assets.sound_5,
				ctx.assets.sound_6,
				ctx.assets.sound_7,
				ctx.assets.sound_8,
				ctx.assets.sound_9,
				ctx.assets.sound_a,
				ctx.assets.sound_b,
				ctx.assets.sound_c,
				ctx.assets.sound_d,
				ctx.assets.sound_e,
				ctx.assets.sound_f,
			];

			let byte = state.emu.program[self.cur_nibble_idx / 2];

			self.cur_pronounce = 0;
			self.cur_nibble = if self.cur_nibble_idx % 2 == 0 {
				(byte & 0xf0) >> 4
			} else {
				byte & 0x0f
			};

			ctx.audio.play(sounds[self.cur_nibble as usize]);

			self.cur_nibble_idx += 1;
			self.is_pronouncing = true;
			self.nibble_timer.start();
		}
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId, state: &PickerState) {
		// Draw window frame
		let win_size = ctx.assets.speaker_window.size;
		Sprite::from(&ctx.assets.speaker_window)
			.with_pos(Self::POS)
			.draw(&mut ctx.painter, canvas);

		// Draw smol display text
		if let Some(idx) = state.equiped_idx {
			let clip = Rect::new_xywh(Self::POS.x + 144.0, Self::POS.y + 11.0, 123.0, 14.0);

			ctx.painter.set_clip(Some(clip));

			let game = &GAMES[idx];
			Text::new(&ctx.assets.w98_font)
				.with_pos(clip.pos + Point::new(2.0, 0.0))
				.with_fg(Color::new(0.0, 1.0, 0.0))
				.with_bg(Color::TRANSPARENT)
				.draw_chars(&mut ctx.painter, canvas, game.title.as_bytes());

			ctx.painter.set_clip(None);
		}

		// Draw window buttons
		self.draw_buttons(ctx, canvas);

		// Draw speaker head
		let speaker_size = ctx.assets.speaker.size;
		Sprite::from(&ctx.assets.speaker)
			.with_frame((self.speaker_frame, 0))
			.with_pos((
				(Self::POS.x + win_size.x / 2.0 - speaker_size.x / 2.0).floor(),
				Self::POS.y + 78.0,
			))
			.draw(&mut ctx.painter, canvas);
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

				if ctx.input.left_just_pressed() {
					clicked = true;
					ctx.audio.play(ctx.assets.button_press_sound);
				}
			}

			button.draw(&mut ctx.painter, canvas);
			clicked
		};

		// Play button
		if draw(0, Self::POS + Point::new(26.0, 10.0)) {
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
