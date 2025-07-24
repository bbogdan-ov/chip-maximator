use crate::{
	app::AppContext,
	math::{Color, Point},
	painter::{CanvasId, Sprite, TextureOpts},
	util::Anim,
};

/// Back board movie display
pub struct MovieDisplay {
	pub canvas: CanvasId,

	anim: Anim,
}
impl MovieDisplay {
	const SIZE: Point = Point::new(154.0, 128.0);

	pub fn new(ctx: &mut AppContext) -> Self {
		Self {
			canvas: ctx.painter.context.new_canvas(
				Self::SIZE,
				Color::BLACK,
				TextureOpts {
					alpha: false,
					min_nearest: false,
					mag_nearest: false,
				},
			),

			anim: Anim::new(8, 0..7).with_looped().with_playing(),
		}
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		self.anim.update(&ctx.time);

		Sprite::from(&ctx.assets.movie)
			.with_anim(&self.anim)
			.draw(&mut ctx.painter, self.canvas);
	}
}
