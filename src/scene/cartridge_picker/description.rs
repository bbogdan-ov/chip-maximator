use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	games::GAMES,
	math::{Color, Point, ToStrBytes},
	painter::{CanvasId, Sprite, Text},
};

use super::PickerState;

/// Cartridge description display
pub struct Description {}
impl Description {
	const POS: Point = Point::new(CANVAS_WIDTH - 300.0 - 30.0, CANVAS_HEIGHT - 300.0 - 30.0);

	pub fn new() -> Self {
		Self {}
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
