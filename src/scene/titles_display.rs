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

/// Titles context
pub(super) struct TitlesContext {
	prev_screen: Option<Screen>,
	cur_screen: Screen,
}
impl TitlesContext {
	pub fn goto_screen(&mut self, screen: Screen) {
		self.prev_screen = Some(self.cur_screen);
		self.cur_screen = screen;
	}
}

/// Back board titles display
pub struct TitlesDisplay {
	context: TitlesContext,

	pub canvas: CanvasId,
	/// Last drawn frame of the previous screen
	prev_screen_canvas: CanvasId,

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
		let size = Point::new(Self::SIZE, Self::SIZE);
		let opts = TextureOpts {
			alpha: false,
			min_nearest: false,
			mag_nearest: false,
		};

		Self {
			context: TitlesContext {
				prev_screen: None,
				cur_screen: Screen::default(),
			},

			canvas: ctx.painter.context.new_canvas(size, Color::BLACK, opts),
			prev_screen_canvas: ctx.painter.context.new_canvas_no_clear(size, opts),

			scoloc: Scoloc::new(ctx),
			titles: Titles::new(ctx),
		}
	}

	pub fn update(&mut self, ctx: &mut AppContext) {
		ctx.input.set_cur_mouse_transform(Self::OFFSET, Self::SCALE);

		match self.context.cur_screen {
			Screen::Titles => (),
			Screen::Scoloc => self.scoloc.update(ctx),
		}

		ctx.input.reset_cur_mouse_transform();
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		ctx.input.set_cur_mouse_transform(Self::OFFSET, Self::SCALE);

		match self.context.cur_screen {
			Screen::Titles => self.titles.draw(ctx, self.canvas, &mut self.context),
			Screen::Scoloc => {
				self.scoloc.offscreen_draw(ctx);
				self.scoloc.draw(ctx, self.canvas, &mut self.context);
			}
		}

		ctx.input.reset_cur_mouse_transform();
	}
}
