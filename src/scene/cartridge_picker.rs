mod carousel;
mod speaker;

use carousel::Carousel;
use speaker::Speaker;

use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	painter::{CanvasId, Sprite},
	state::State,
};

/// Cartridge picker
pub struct CartridgePicker {
	carousel: Carousel,
	speaker: Speaker,
}
impl Default for CartridgePicker {
	fn default() -> Self {
		Self {
			carousel: Carousel::new(),
			speaker: Speaker::new(),
		}
	}
}
impl CartridgePicker {
	pub fn update(&mut self, ctx: &mut AppContext, state: &State) {
		self.carousel.update(ctx);
		self.speaker.update(ctx, state);
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		// Draw darken rect
		Sprite::new(ctx.painter.white_texture, (CANVAS_WIDTH, CANVAS_HEIGHT))
			.with_fg((0.0, 0.0, 0.0))
			.draw(&mut ctx.painter, canvas);

		self.carousel.draw(ctx, canvas);
		self.speaker.draw(ctx, canvas);
	}
}
