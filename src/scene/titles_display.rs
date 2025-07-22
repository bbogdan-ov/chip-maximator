mod scoloc;
mod titles;

use scoloc::*;
use titles::*;

use crate::{
	app::AppContext,
	math::{Color, Point},
	painter::{CanvasId, TextureOpts},
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
	scoloc: Scoloc,
	titles: Titles,
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
			scoloc: Scoloc::new(ctx),
			titles: Titles::new(ctx),
		}
	}

	pub fn update(&mut self, ctx: &mut AppContext) {
		ctx.input.set_cur_mouse_transform(Self::OFFSET, Self::SCALE);

		match self.cur_screen {
			Screen::Titles => (),
			Screen::Scoloc => self.scoloc.update(ctx),
		}

		ctx.input.reset_cur_mouse_transform();
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		ctx.input.set_cur_mouse_transform(Self::OFFSET, Self::SCALE);

		match self.cur_screen {
			Screen::Titles => self.titles.draw(ctx, self.canvas, &mut self.cur_screen),
			Screen::Scoloc => {
				self.scoloc.offscreen_draw(ctx);
				self.scoloc.draw(ctx, self.canvas, &mut self.cur_screen);
			}
		}

		ctx.input.reset_cur_mouse_transform();
	}
}
