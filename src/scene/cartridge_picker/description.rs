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
	"uncontrollable crowd",
	"where were these people?",
	"hard to laugh",
	"there is no reason",
	"3 meters underground",
	"misspelled jokes",
	"fixing broken things",
	"drinking forbidden drinks",
	"nonsensical words",
	"where are those clowns?",
	"closing doors",
	"baking bread",
	"finding a purpose",
	"forgetting bad things",
	"disappointed child",
	"hello kids",
	"people who love eachother",
	"cherry picking",
	"come on",
	"atomic operations",
	"binary shit",
	"waiting for a bus",
	"being infinitely thankful",
];

const MAX_TYPO_LEN: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WriterState {
	Nothing,
	Typing,
	MakingTypo { one_bad_letter: bool, len: usize },
	ErasingTypo,
	ErasingEverything,
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

	cur_game_idx: Option<usize>,

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

			cur_game_idx: None,

			typing_timer: Timer::from_millis(100),
			cursor_blink_timer: Timer::from_millis(300),
		}
	}

	fn start_typing_timer(&mut self, scale: u64) {
		self.start_typing_timer_from_to(20 * scale, 60 * scale);
	}
	fn start_typing_timer_from_to(&mut self, low: u64, high: u64) {
		let millis = quad_rand::gen_range::<u64>(low, high);
		self.typing_timer.start_millis(millis);
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
			if let Some(ch) = game.desc.as_bytes().get(idx) {
				typo_char = *ch;
			} else {
				typo_char = rand_char(LOWER_ALPHA_LETTERS, next_char);
			}
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

	fn start_typing(&mut self, idx: usize) {
		let game = &GAMES[idx];
		if game.desc.is_empty() {
			return;
		}

		self.cur_game_idx = Some(idx);
		self.char_idx = 0;
		self.writer_state = WriterState::Typing;
	}
	fn erase_everything(&mut self) {
		if self.writer_state == WriterState::ErasingEverything {
			return;
		}

		self.writer_state = WriterState::ErasingEverything;
		self.start_typing_timer(8);
	}

	fn update_typing(&mut self, picker: &PickerState) {
		let Some(game_idx) = self.cur_game_idx else {
			return;
		};
		let game = &GAMES[game_idx];

		if self.writer_state == WriterState::Nothing || !self.typing_timer.finished() {
			return;
		}

		if self.writer_state != WriterState::ErasingEverything
			&& self.char_idx + 1 >= game.desc.len()
		{
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
				let one_bad_letter = quad_rand::rand() % 100 <= 80;

				if next_char.is_ascii_alphabetic() {
					if one_bad_letter {
						len = quad_rand::gen_range(1, MAX_TYPO_LEN);
					} else {
						len = quad_rand::gen_range(1, 3);
					}
				} else {
					len = 1;
				}

				len = usize::min(self.char_idx + len, game.desc.len()) - self.char_idx;

				self.writer_state = WriterState::MakingTypo {
					one_bad_letter,
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
				let one_bad = *one_bad_letter;

				if self.typo_len >= *len {
					self.writer_state = WriterState::ErasingTypo;
					self.start_typing_timer(6);
				} else {
					self.advance_typo(game, next_char, one_bad && self.typo_len >= 1);
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

			WriterState::ErasingEverything => {
				self.start_typing_timer_from_to(10, 40);

				if self.typo_len > 0 {
					self.typo_len -= 1;
				} else if self.char_idx > 0 {
					self.char_idx -= 1;
				} else {
					self.writer_state = WriterState::Nothing;
					self.cur_game_idx = None;

					if let Some(equiped_idx) = picker.equiped_idx {
						self.start_typing(equiped_idx);
						self.start_typing_timer(10);
					}
				}

				self.cursor_blink_timer.start();
			}
		}
	}

	pub fn update(&mut self, ctx: &AppContext, picker: &PickerState) {
		self.typing_timer.update(&ctx.time);
		self.cursor_blink_timer.update(&ctx.time);

		if picker.just_equipped {
			let Some(equiped_idx) = picker.equiped_idx else {
				return;
			};

			if let Some(cur_game_idx) = self.cur_game_idx {
				if cur_game_idx == equiped_idx {
					self.writer_state = WriterState::Typing;
					self.start_typing_timer(10);
				} else {
					self.erase_everything();
				}
			} else {
				self.start_typing(equiped_idx);
			}
		} else if picker.just_unequipped {
			self.erase_everything();
		}

		self.update_typing(picker);
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
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

		let text_pos = Self::POS + Point::new(12.0, 33.0);
		let mut num_lines: u32 = 0;
		let mut num_chars: u32 = 0;
		let mut cursor_pos = text_pos;

		if let Some(game_idx) = self.cur_game_idx {
			let game = &GAMES[game_idx];

			// Draw text
			let mut text = Text::new(&ctx.assets.serif_font)
				.with_pos(text_pos)
				.with_font_size(0.5)
				.with_fg(Color::BLACK)
				.with_bg(Color::TRANSPARENT);
			let desc_text_slice = &game.desc.as_bytes()[..self.char_idx];

			text.draw_str(&mut ctx.painter, canvas, desc_text_slice);

			if self.typo_len > 0 {
				text.draw_str(&mut ctx.painter, canvas, &self.typo[..self.typo_len]);
			}

			let char_size = text.char_size();
			cursor_pos = Point::new(
				text.pos.x + text.char_offset_px,
				text.pos.y + text.line_offset * char_size.y,
			);

			num_lines = text.line_offset as u32;
			num_chars = desc_text_slice.len() as u32 + self.typo_len as u32;
		}

		// Draw text cursor
		if !self.cursor_blink_timer.finished() || ctx.time.elapsed % 60 < 30 {
			Sprite::new(ctx.painter.white_texture, Point::new(2.0, 18.0))
				.with_fg(Color::BLACK)
				.with_pos(cursor_pos)
				.draw(&mut ctx.painter, canvas);
		}

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

	fn next_char(&self, game: &GameInfo) -> u8 {
		game.desc.as_bytes()[self.char_idx]
	}
}
