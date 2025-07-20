mod scoundrel;

use scoundrel::*;

use crate::{
	app::AppContext,
	math::{Color, Point},
	painter::{CanvasId, Sprite, TextureOpts},
};

/// Back board titles display
pub struct TitlesDisplay {
	pub canvas: CanvasId,

	scoundrel: Scoundrel,
}
impl TitlesDisplay {
	const SIZE: f32 = 256.0;

	/// Hard-coded top-left position of the display relative to the window top-left corner
	/// There is no way i could (while maintaining performance) calculate the offset
	const OFFSET: Point = Point::new(144.0, 136.0);
	/// Hard-coded and randomly picked canvas scale relative to the window
	const SCALE: f32 = 0.77;

	pub fn new(ctx: &mut AppContext) -> Self {
		Self {
			canvas: ctx.painter.context.new_canvas(
				(Self::SIZE, Self::SIZE),
				Color::BLACK,
				TextureOpts {
					alpha: false,
					min_nearest: false,
					mag_nearest: false,
				},
			),

			scoundrel: Scoundrel::new(ctx),
		}
	}

	pub fn update(&mut self, ctx: &mut AppContext) {
		self.scoundrel.update(ctx);
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		Sprite::from(&ctx.assets.titles_bg).draw(&mut ctx.painter, self.canvas);

		ctx.input.set_cur_mouse_transform(Self::OFFSET, Self::SCALE);
		self.scoundrel.draw(ctx, self.canvas);
		ctx.input.reset_cur_mouse_transform();
	}
}
