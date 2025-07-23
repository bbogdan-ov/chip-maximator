mod scoloc;
mod titles;

use std::time::Duration;

use scoloc::*;
use titles::*;

use crate::{
	app::AppContext,
	math::Point,
	painter::{CanvasId, Sprite, TextureOpts},
	util::{Easing, Tweenable},
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
	screen_switched: bool,
	cur_screen: Screen,
}
impl TitlesContext {
	pub fn goto_screen(&mut self, screen: Screen) {
		self.screen_switched = true;
		self.cur_screen = screen;
	}
}

/// Back board titles display
pub struct TitlesDisplay {
	context: TitlesContext,

	/// The first canvas is the current one and the second
	/// one retains last frame of the previous screen
	canvases: [CanvasId; 2],
	transition_tween: Tweenable,

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
				screen_switched: false,
				cur_screen: Screen::default(),
			},

			canvases: [
				ctx.painter.context.new_canvas_no_clear(size, opts),
				ctx.painter.context.new_canvas_no_clear(size, opts),
			],
			transition_tween: Tweenable::new(1.0),

			scoloc: Scoloc::new(ctx),
			titles: Titles::new(ctx),
		}
	}

	pub fn update(&mut self, ctx: &mut AppContext) {
		self.transition_tween.update(&ctx.time);

		ctx.input.set_cur_mouse_transform(Self::OFFSET, Self::SCALE);

		match self.context.cur_screen {
			Screen::Titles => (),
			Screen::Scoloc => self.scoloc.update(ctx),
		}

		if self.context.screen_switched {
			self.canvases.swap(0, 1);
			self.context.screen_switched = false;

			let dur = Duration::from_millis(500);
			self.transition_tween
				.play_from(0.0, 1.0, dur, Easing::InSine);
		}

		ctx.input.reset_cur_mouse_transform();
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		ctx.input.set_cur_mouse_transform(Self::OFFSET, Self::SCALE);

		self.draw_screen(ctx, self.canvases[0], self.context.cur_screen);

		// Draw animated previous screen
		let p = self.transition_tween.value;
		Sprite::from(ctx.painter.canvas(self.canvases[1]))
			.with_pos((0.0, Self::SIZE * p))
			.draw(&mut ctx.painter, self.canvases[0]);

		ctx.input.reset_cur_mouse_transform();
	}
	fn draw_screen(&mut self, ctx: &mut AppContext, canvas: CanvasId, screen: Screen) {
		match screen {
			Screen::Titles => self.titles.draw(ctx, canvas, &mut self.context),
			Screen::Scoloc => {
				self.scoloc.offscreen_draw(ctx);
				self.scoloc.draw(ctx, canvas, &mut self.context);
			}
		}
	}

	pub fn cur_canvas(&self) -> CanvasId {
		self.canvases[0]
	}
}
