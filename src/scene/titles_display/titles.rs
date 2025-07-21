use crate::{
	app::AppContext,
	math::Color,
	painter::{CanvasId, Sprite, Text},
};

/// Titles
pub struct Titles {}
impl Titles {
	pub fn new() -> Self {
		Self {}
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		// Draw background image
		Sprite::from(&ctx.assets.titles_bg).draw(&mut ctx.painter, canvas);

		Text::new(&ctx.assets.serif_font)
			.with_pos((8.0, 8.0))
			.with_fg(Color::BLACK)
			.with_bg(Color::TRANSPARENT)
			.draw_chars(&mut ctx.painter, canvas, b"Thank you");
	}
}
