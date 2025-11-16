use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	assets::{DIGIT_LETTERS, LOWER_ALPHA_LETTERS, SYMBOL_LETTERS, UPPER_ALPHA_LETTERS},
	games::{GAMES, GameInfo},
	math::{Color, Point, Rect, ToStrBytes},
	painter::{CanvasId, Sprite, Text},
	util::Timer,
};

use super::PickerState;

const NONSENSE: &[&str] = &[
	"a way home",
	"walking alone",
	"being happy",
	"waiting for something to happen",
	"unreasonably complex thinking",
	"being sad",
	"cake bakery",
	"inventing a new disease",
	"wheel rolling",
	"making fun of people",
	"punching someones ass",
	"chip maximazing",
	"drowning kitties",
	"typo correction",
	"expression evaluation",
	"smelly nerds",
	"job",
	"broken spine",
	"love eachother",
	"stop disturbing",
	"kingdom is crashing",
	"making buildings from legos",
	"it is not written in russian",
	"crash out",
	"tall tales",
	"talking bullshit",
	"fuck around",
	"being productive",
	"an ability to reproduce",
	"free people",
	"dedicated to all human beings",
	"worldwide confusion",
	"phrase translation",
	"repeating sentences",
	"throwing stones at eachother",
];

const MAX_TYPO_LEN: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WriterState {
	Nothing,
	Typing,
	MakingTypo { one_bad_letter: bool, len: usize },
	ErasingTypo,
}

/// Cartridge description display
pub struct Description {
	nonsense_idx: usize,
	nonsense_offset: f32,

	writer_state: WriterState,
	typo: [u8; MAX_TYPO_LEN],
	typo_len: usize,
	char_idx: usize,
	was_typing_newlines: bool,

	typing_timer: Timer,
	cursor_blink_timer: Timer,
}
impl Description {
	const POS: Point = Point::new(CANVAS_WIDTH - 300.0 - 30.0, CANVAS_HEIGHT - 300.0 - 30.0);

	pub fn new() -> Self {
		Self {
			nonsense_idx: 0,
			nonsense_offset: 0.0,

			writer_state: WriterState::Nothing,
			typo: [0; MAX_TYPO_LEN],
			typo_len: 0,
			char_idx: 0,
			was_typing_newlines: false,

			typing_timer: Timer::from_millis(100),
			cursor_blink_timer: Timer::from_millis(300),
		}
	}

	fn start_typing_timer(&mut self, scale: u64) {
		let millis = quad_rand::gen_range::<u64>(20, 60);
		self.typing_timer.start_millis(millis * scale);
	}

	fn advance_typing(&mut self) {
		self.char_idx += 1;
	}
	fn advance_typo(&mut self, game: &GameInfo, next_char: u8, copy_next_chars: bool) {
		let typo_char: u8;

		fn rand_char(set: &str, not_eq: u8) -> u8 {
			let mut idx;
			loop {
				idx = quad_rand::rand() as usize % set.len();
				if set.as_bytes()[idx] != not_eq {
					break;
				}
			}
			set.as_bytes()[idx]
		}

		if copy_next_chars {
			let idx = self.char_idx + self.typo_len;
			typo_char = game.desc.as_bytes()[idx];
		} else if next_char.is_ascii_uppercase() {
			typo_char = rand_char(UPPER_ALPHA_LETTERS, next_char);
		} else if next_char.is_ascii_lowercase() {
			typo_char = rand_char(LOWER_ALPHA_LETTERS, next_char);
		} else if next_char.is_ascii_digit() {
			typo_char = rand_char(DIGIT_LETTERS, next_char);
		} else {
			typo_char = rand_char(SYMBOL_LETTERS, next_char);
		}

		self.typo[self.typo_len] = typo_char;
		self.typo_len += 1;
	}

	fn start_typing(&mut self) {
		self.char_idx = 0;
		self.writer_state = WriterState::Typing;
	}

	fn update_typing(&mut self, game: &GameInfo) {
		if self.writer_state == WriterState::Nothing || !self.typing_timer.finished() {
			return;
		}

		if self.char_idx + 1 >= game.desc.len() {
			self.writer_state = WriterState::Nothing;
			return;
		}

		self.cursor_blink_timer.start();

		let next_char = self.next_char(game);
		let should_typo = quad_rand::rand() % 1000 <= 40; // 4%

		match &mut self.writer_state {
			WriterState::Nothing => unreachable!(),

			WriterState::Typing if next_char == b'\n' => {
				self.advance_typing();
				self.start_typing_timer(3);
				self.was_typing_newlines = true;
			}
			WriterState::Typing if next_char == b' ' => {
				self.advance_typing();
				self.start_typing_timer(2);
			}
			WriterState::Typing if should_typo => {
				let mut len: usize;
				let one_bad = quad_rand::rand() % 100 <= 60;

				if next_char.is_ascii_alphabetic() {
					len = quad_rand::gen_range(1, MAX_TYPO_LEN);
				} else {
					len = 1;
				}

				if one_bad {
					len = usize::min(self.char_idx + len, game.desc.len()) - self.char_idx;
				}

				self.writer_state = WriterState::MakingTypo {
					one_bad_letter: one_bad,
					len,
				};
				self.start_typing_timer(1);
			}
			WriterState::Typing => {
				self.advance_typing();
				if self.was_typing_newlines {
					self.start_typing_timer(6);
					self.was_typing_newlines = false;
				} else {
					self.start_typing_timer(1);
				}
			}

			WriterState::MakingTypo {
				one_bad_letter,
				len,
			} => {
				let copy_next = !*one_bad_letter;
				*one_bad_letter = false;

				if self.typo_len >= *len {
					self.writer_state = WriterState::ErasingTypo;
					self.start_typing_timer(6);
				} else {
					self.advance_typo(game, next_char, copy_next);
					self.start_typing_timer(1);
				}
			}

			WriterState::ErasingTypo => {
				if self.typo_len == 0 {
					self.writer_state = WriterState::Typing;
					self.start_typing_timer(6);
				} else {
					self.typo_len -= 1;
					self.start_typing_timer(2);
				}
			}
		}
	}

	pub fn update(&mut self, ctx: &AppContext, picker: &PickerState) {
		self.typing_timer.update(&ctx.time);
		self.cursor_blink_timer.update(&ctx.time);

		let Some(equiped_idx) = picker.equiped_idx else {
			return;
		};
		let equiped_game = &GAMES[equiped_idx];

		if picker.just_equipped && !equiped_game.desc.is_empty() {
			self.start_typing();
		}

		self.update_typing(equiped_game);
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId, picker: &PickerState) {
		// Draw window
		Sprite::from(&ctx.assets.description_window)
			.with_pos(Self::POS)
			.draw(&mut ctx.painter, canvas);

		// Draw diode
		Sprite::from(&ctx.assets.description_diode)
			.with_pos(Self::POS + Point::new(274.0, 0.0))
			.with_opacity(((ctx.time.elapsed as f32 / 30.0).sin() + 1.0) / 2.0)
			.draw(&mut ctx.painter, canvas);

		// Draw nonsense display
		{
			let rect = Rect::new_xywh(Self::POS.x + 171.0, Self::POS.y + 279.0, 124.0, 16.0);

			ctx.painter.set_clip(Some(rect));

			let mut text = Text::new(&ctx.assets.w98_font)
				.with_pos(rect.pos)
				.with_fg(Color::new(0.0, 1.0, 0.0))
				.with_bg(Color::TRANSPARENT);

			let sentence = NONSENSE[self.nonsense_idx];
			let width = text.measure_chars(sentence.as_bytes());

			self.nonsense_offset -= 2.0;
			if self.nonsense_offset <= -width {
				self.nonsense_offset = rect.size.x + 2.0;

				let mut new_idx = self.nonsense_idx;
				while new_idx == self.nonsense_idx {
					new_idx = quad_rand::rand() as usize % NONSENSE.len();
				}
				self.nonsense_idx = new_idx;
			}

			text.char_offset_px = 0.0;
			text.pos.x = rect.pos.x + self.nonsense_offset;
			text.draw_chars(&mut ctx.painter, canvas, sentence.as_bytes());

			ctx.painter.set_clip(None);
		}

		if let Some(idx) = picker.equiped_idx {
			let game = &GAMES[idx];

			// Draw text
			let mut text = Text::new(&ctx.assets.serif_font)
				.with_pos(Self::POS + Point::new(12.0, 33.0))
				.with_font_size(0.5)
				.with_fg(Color::BLACK)
				.with_bg(Color::TRANSPARENT);

			text.draw_str(
				&mut ctx.painter,
				canvas,
				game.desc[..self.char_idx].as_bytes(),
			);

			if self.typo_len > 0 {
				text.draw_str(&mut ctx.painter, canvas, &self.typo[..self.typo_len]);
			}

			// Draw text cursor
			if !self.cursor_blink_timer.finished() || ctx.time.elapsed % 60 < 30 {
				let char_size = text.char_size();
				let pos = Point::new(
					text.pos.x + text.char_offset_px,
					text.pos.y + text.line_offset * char_size.y,
				);

				Sprite::new(
					ctx.painter.white_texture,
					Point::new(2.0, char_size.y - 4.0),
				)
				.with_fg(Color::BLACK)
				.with_pos(pos)
				.draw(&mut ctx.painter, canvas);
			}

			let num_lines = text.line_offset as u32;
			let num_chars = game.desc.len() as u32;

			// Draw footer text
			{
				let mut text = Text::new(&ctx.assets.w98_font)
					.with_fg(Color::BLACK)
					.with_bg(Color::TRANSPARENT)
					.with_pos(Self::POS + Point::new(6.0, 280.0));

				// Lines number
				text.draw_chars(&mut ctx.painter, canvas, &num_lines.to_str_bytes())
					.draw_chars(&mut ctx.painter, canvas, b" line(s)");

				// Chars number
				text.char_offset_px = 0.0;
				text.pos.x = Self::POS.x + 90.0;
				text.draw_chars(&mut ctx.painter, canvas, &num_chars.to_str_bytes())
					.draw_chars(&mut ctx.painter, canvas, b" char(s)");
			}
		}
	}

	fn next_char(&self, game: &GameInfo) -> u8 {
		game.desc.as_bytes()[self.char_idx + 1]
	}
}
