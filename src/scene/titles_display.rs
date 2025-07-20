use crate::{
	app::AppContext,
	math::Color,
	painter::{CanvasId, Sprite, Text, TextureOpts},
};

/// Back board titles display
pub struct TitlesDisplay {
	pub canvas: CanvasId,
}
impl TitlesDisplay {
	const SIZE: f32 = 256.0;

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
		}
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		Sprite::from(&ctx.assets.titles_bg).draw(&mut ctx.painter, self.canvas);

		Text::new(&ctx.assets.serif_font)
			.with_pos((10.0, 10.0))
			.with_fg(Color::BLACK)
			.with_bg(Color::TRANSPARENT)
			.draw_line(&mut ctx.painter, self.canvas, b"Thank you");

		Text::new(&ctx.assets.ibm_font)
			.with_pos((10.0, 50.0))
			.draw_line(&mut ctx.painter, self.canvas, b"Thank you");
	}
}
