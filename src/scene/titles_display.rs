mod scoloc;

use scoloc::*;

use crate::{
	app::AppContext,
	math::{Color, Point},
	painter::{CanvasId, Sprite, Text, TextureOpts},
};

/// Screen
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum Screen {
	#[default]
	Titles,
	Scoloc,
}

/// Back board titles display
pub struct TitlesDisplay {
	pub canvas: CanvasId,

	cur_screen: Screen,
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

			cur_screen: Screen::default(),
			scoundrel: Scoundrel::new(ctx),
		}
	}

	pub fn update(&mut self, ctx: &mut AppContext) {
		ctx.input.set_cur_mouse_transform(Self::OFFSET, Self::SCALE);

		match self.cur_screen {
			Screen::Titles => (),
			Screen::Scoloc => self.scoundrel.update(ctx),
		}

		ctx.input.reset_cur_mouse_transform();
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		ctx.input.set_cur_mouse_transform(Self::OFFSET, Self::SCALE);

		match self.cur_screen {
			Screen::Titles => self.draw_titles(ctx, self.canvas),
			Screen::Scoloc => {
				self.scoundrel.offscreen_draw(ctx);
				self.scoundrel.draw(ctx, self.canvas, &mut self.cur_screen);
			}
		}

		ctx.input.reset_cur_mouse_transform();
	}

	fn draw_titles(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		// Draw background image
		Sprite::from(&ctx.assets.titles_bg).draw(&mut ctx.painter, self.canvas);

		Text::new(&ctx.assets.serif_font)
			.with_pos((8.0, 8.0))
			.with_fg(Color::BLACK)
			.with_bg(Color::TRANSPARENT)
			.draw_chars(&mut ctx.painter, canvas, b"Thank you");
	}
}
