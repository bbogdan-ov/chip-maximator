use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	games::GAMES,
	math::{Color, Point},
	painter::{CanvasId, Sprite, Text},
};

use super::PickerState;

/// Cartridge description display
pub struct Description {
	canvas: CanvasId,

	/// Whether the "source code" tab is the current one
	is_code_tab: bool,
}
impl Description {
	const POS: Point = Point::new(CANVAS_WIDTH - 460.0 - 30.0, CANVAS_HEIGHT - 300.0 - 30.0);
	const SIZE: Point = Point::new(292.0, 256.0);

	pub fn new(ctx: &mut AppContext) -> Self {
		Self {
			canvas: ctx.painter.context.new_canvas(
				Self::SIZE,
				Color::TRANSPARENT,
				Default::default(),
			),

			is_code_tab: false,
		}
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext, state: &PickerState) {
		// Draw text
		if let Some(idx) = state.equiped_idx {
			let game = &GAMES[idx];
			Text::new(&ctx.assets.serif_font)
				.with_pos((14.0, 14.0))
				.with_font_size(0.7)
				.with_fg(Color::hex(0x4ad4ec))
				.with_bg(Color::TRANSPARENT)
				.draw_str(&mut ctx.painter, self.canvas, game.desc);
		}
	}
	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		// Draw window
		Sprite::from(&ctx.assets.desc_display)
			.with_pos(Self::POS)
			.draw(&mut ctx.painter, canvas);

		let cvs = ctx.painter.canvas(self.canvas);
		let pos = Self::POS + Point::new(148.0, 28.0);
		Sprite::new(cvs.texture, cvs.size())
			.with_pos(pos)
			.draw(&mut ctx.painter, canvas);

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
