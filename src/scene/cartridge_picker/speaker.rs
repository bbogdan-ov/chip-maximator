use std::time::Duration;

use crate::{
	app::{AppContext, CANVAS_WIDTH},
	games::GAMES,
	math::{Color, Point, Rect},
	painter::{CanvasId, Sprite, Text},
	state::State,
	util::{Easing, Timer, Tweenable},
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

/// Speaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpeakerState {
	Sleeping,
	Talking,
	Suffering,
}

/// Speaker
/// Reads and speaks cartridge program bytes out loud
pub struct Speaker {
	state: SpeakerState,

	cur_nibble: u8,
	/// Current unsigned 4-bit int index in cartridge ROM
	cur_nibble_idx: usize,
	cur_pronounce: usize,
	is_pronouncing: bool,

	/// Title typing animation tween
	title_tween: Tweenable,
	/// Delay between nibble pronounciation
	pronounce_nibble_timer: Timer,
	/// Delay between pronounce animation frames
	pronounce_frame_timer: Timer,
	/// Delay before going to sleep
	rest_timer: Timer,
	suffering_timer: Timer,

	cur_frame: i32,
}
impl Speaker {
	const POS: Point = Point::new(CANVAS_WIDTH - 320.0, 30.0);

	pub fn new() -> Self {
		Self {
			state: SpeakerState::Sleeping,

			cur_nibble: 0,
			cur_nibble_idx: 0,
			cur_pronounce: 0,
			is_pronouncing: false,

			title_tween: Tweenable::default(),
			pronounce_nibble_timer: Timer::from_millis(1000),
			pronounce_frame_timer: Timer::from_millis(100),
			rest_timer: Timer::from_millis(1500),
			suffering_timer: Timer::from_millis(200),

			cur_frame: 0,
		}
	}

	pub fn update(&mut self, ctx: &mut AppContext, state: &State, picker: &PickerState) {
		self.title_tween.update(&ctx.time);
		self.pronounce_nibble_timer.update(&ctx.time);
		self.pronounce_frame_timer.update(&ctx.time);
		self.rest_timer.update(&ctx.time);
		self.suffering_timer.update(&ctx.time);

		if picker.just_equipped {
			let dur = Duration::from_millis(1000);
			self.title_tween.play_from(0.0, 1.0, dur, Easing::Linear);
		}

		if picker.just_unequipped && self.state == SpeakerState::Talking {
			self.suffering_timer.start();
			self.state = SpeakerState::Suffering;

			self.pronounce_frame_timer.stop();
			self.is_pronouncing = false;

			ctx.audio.play_random(
				&ctx.time,
				&[
					ctx.assets.suffering_1_sound,
					ctx.assets.suffering_2_sound,
					ctx.assets.suffering_3_sound,
				],
			);
		}

		match self.state {
			SpeakerState::Sleeping => {
				self.cur_nibble_idx = 0;

				let p = self.rest_timer.progress();
				if p >= 1.0 {
					// Fully close eyes
					self.cur_frame = 0;
				} else if p > 0.9 {
					// Close eyes in half
					self.cur_frame = 1;
				}
			}
			SpeakerState::Talking => self.update_talking(ctx, state),
			SpeakerState::Suffering => {
				if ctx.time.elapsed % 2 == 0 {
					self.cur_frame = quad_rand::gen_range(8, 15);
				}
				if self.suffering_timer.finished() {
					self.rest_timer.stop();
					self.state = SpeakerState::Sleeping;
				}
			}
		}

		if self.pronounce_frame_timer.finished() && self.is_pronouncing {
			let pronounces = LETTERS_PRONOUNCE[self.cur_nibble as usize];
			if self.cur_pronounce >= pronounces.len() {
				self.cur_frame = 2;
				self.is_pronouncing = false;
				return;
			}

			match pronounces[self.cur_pronounce] {
				Pronounce::A => self.cur_frame = 3,
				Pronounce::AAA => self.cur_frame = 4,
				Pronounce::OOO => self.cur_frame = 5,
				Pronounce::III => self.cur_frame = 6,
				Pronounce::FFF => self.cur_frame = 7,
			}

			if self.state == SpeakerState::Talking || self.cur_pronounce > 0 {
				self.cur_pronounce += 1;
				self.pronounce_frame_timer.start();
				self.rest_timer.start();
			}
		}
	}
	fn update_talking(&mut self, ctx: &mut AppContext, state: &State) {
		let Some(program) = state.emu.program else {
			return;
		};

		if self.pronounce_nibble_timer.finished() {
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

			let byte = program[self.cur_nibble_idx / 2];

			self.cur_pronounce = 0;
			self.cur_nibble = if self.cur_nibble_idx % 2 == 0 {
				(byte & 0xf0) >> 4
			} else {
				byte & 0x0f
			};

			ctx.audio.play(sounds[self.cur_nibble as usize]);

			self.cur_nibble_idx += 1;
			self.is_pronouncing = true;
			self.pronounce_nibble_timer.start();
		}
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId, state: &PickerState) {
		let mut win_pos = Self::POS;

		if self.state == SpeakerState::Suffering {
			// Shake the window
			win_pos.x += quad_rand::gen_range(-2.0, 2.0);
		}

		// Draw window frame
		let win_size = ctx.assets.speaker_window.size;
		Sprite::from(&ctx.assets.speaker_window)
			.with_pos(win_pos)
			.draw(&mut ctx.painter, canvas);

		// Draw smol display text
		if let Some(idx) = state.equiped_idx {
			let clip = Rect::new_xywh(win_pos.x + 144.0, win_pos.y + 11.0, 123.0, 14.0);

			ctx.painter.set_clip(Some(clip));

			let game = &GAMES[idx];
			let mut text = Text::new(&ctx.assets.w98_font)
				.with_pos(clip.pos + Point::new(2.0, 0.0))
				.with_fg(Color::new(0.0, 1.0, 0.0))
				.with_bg(Color::TRANSPARENT);

			text.begin_draw(&mut ctx.painter, canvas);
			let bytes = game.title.as_bytes();
			for (idx, byte) in bytes.iter().enumerate() {
				const LETTERS: &str =
					"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,-!?/#";

				let f = idx as f32 / bytes.len() as f32;

				let b = if f <= self.title_tween.value {
					*byte
				} else {
					let i = (ctx.time.elapsed as usize * idx) % LETTERS.len();
					LETTERS.as_bytes()[i]
				};

				text.draw_char(&mut ctx.painter, b, Point::default());
			}

			ctx.painter.set_clip(None);
		}

		// Draw window buttons
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
		if draw(0, win_pos + Point::new(26.0, 10.0)) {
			self.state = SpeakerState::Talking;
		}
		// Stop button
		if draw(1, win_pos + Point::new(52.0, 10.0)) {
			self.state = SpeakerState::Sleeping;
		}

		ctx.painter.set_clip(Some(Rect::new(
			win_pos + Point::new(27.0, 32.0),
			(256.0, 256.0).into(),
		)));

		// Draw window grid
		let offset_x = ctx.time.elapsed as f32 / 2.0 % 32.0;
		let offset = Point::new(27.0 - offset_x.floor(), 160.0);
		Sprite::from(&ctx.assets.speaker_grid)
			.with_pos(win_pos + offset)
			.draw(&mut ctx.painter, canvas);

		// Draw speaker head
		let speaker_size = ctx.assets.speaker.size;
		Sprite::from(&ctx.assets.speaker)
			.with_frame((self.cur_frame, 0))
			.with_pos((
				(win_pos.x + win_size.x / 2.0 - speaker_size.x / 2.0).floor(),
				win_pos.y + 78.0,
			))
			.draw(&mut ctx.painter, canvas);

		ctx.painter.set_clip(None);
	}
}
