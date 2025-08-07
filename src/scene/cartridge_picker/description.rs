use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	math::Point,
	painter::{CanvasId, Sprite},
};

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

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		// Draw window
		Sprite::from(&ctx.assets.desc_display)
			.with_pos(Self::POS)
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
