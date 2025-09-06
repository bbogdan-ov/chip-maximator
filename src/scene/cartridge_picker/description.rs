use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	games::GAMES,
	math::{Color, Point},
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

		// Draw text
		if let Some(idx) = picker.equiped_idx {
			let game = &GAMES[idx];
			Text::new(&ctx.assets.serif_font)
				.with_pos(Self::POS + Point::new(16.0, 33.0))
				.with_font_size(0.5)
				.with_fg(Color::BLACK)
				.with_bg(Color::TRANSPARENT)
				.draw_str(&mut ctx.painter, canvas, game.desc);
		}
	}
}
