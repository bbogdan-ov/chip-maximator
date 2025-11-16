use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	games::GAMES,
	math::{Color, Point, Rect, ToStrBytes},
	painter::{CanvasId, Sprite, Text},
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

/// Cartridge description display
pub struct Description {
	nonsense_idx: usize,
	nonsense_offset: f32,
}
impl Description {
	const POS: Point = Point::new(CANVAS_WIDTH - 300.0 - 30.0, CANVAS_HEIGHT - 300.0 - 30.0);

	pub fn new() -> Self {
		Self {
			nonsense_idx: 0,
			nonsense_offset: 0.0,
		}
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
			text.draw_str(&mut ctx.painter, canvas, game.desc);

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
}
