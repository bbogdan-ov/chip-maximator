use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	games::GAMES,
	math::{Color, Point, Rect},
	painter::{CanvasId, Sprite, Text},
};

use super::PickerState;

/// Cartridge description display
pub struct Description {
	/// Whether the "source code" tab is the current one
	is_code_tab: bool,
}
impl Description {
	const POS: Point = Point::new(CANVAS_WIDTH - 460.0 - 30.0, CANVAS_HEIGHT - 300.0 - 30.0);

	pub fn new() -> Self {
		Self { is_code_tab: false }
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId, state: &PickerState) {
		// Draw window
		Sprite::from(&ctx.assets.desc_display)
			.with_pos(Self::POS)
			.draw(&mut ctx.painter, canvas);

		// Draw text
		if let Some(idx) = state.equiped_idx {
			let clip = Rect::new_xywh(Self::POS.x + 148.0, Self::POS.y + 28.0, 292.0, 256.0);
			ctx.painter.set_clip(Some(clip));

			let game = &GAMES[idx];
			let pos = Self::POS + Point::new(162.0, 42.0);
			Text::new(&ctx.assets.serif_font)
				.with_pos(pos)
				.with_font_size(0.7)
				.with_bg(Color::TRANSPARENT)
				.draw_str(&mut ctx.painter, canvas, game.desc);

			ctx.painter.set_clip(None);
		}

		// Draw tabs
		let pos = Self::POS + Point::new(144.0, 8.0);
		let tabs = Sprite::from(&ctx.assets.desc_display_tabs)
			.with_pos(pos)
			.with_frame((0, self.is_code_tab as i32));

		if tabs.is_hover(&mut ctx.input) && ctx.input.left_just_pressed() {
			self.is_code_tab ^= true;
		}

		tabs.draw(&mut ctx.painter, canvas);
	}
}
