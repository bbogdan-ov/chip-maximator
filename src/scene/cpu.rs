use core::f32;

use crate::{
	app::AppContext,
	painter::{CanvasId, Sprite},
	state::State,
	util::Easing,
};

/// Front board CPU
/// Gets heated when emulator is trapped in endless loop
pub struct Cpu {
	sprite: Sprite,
}
impl Cpu {
	const SHAKE_THRESHOLD: f32 = 20.0;

	pub fn new(ctx: &AppContext) -> Self {
		Self {
			sprite: Sprite::from(&ctx.assets.heated_cpu).with_pos((380.0, 294.0)),
		}
	}

	pub fn update(&mut self, ctx: &mut AppContext, state: &mut State) {
		self.update_shaking(ctx, state);
	}
	fn update_shaking(&self, ctx: &mut AppContext, state: &mut State) {
		let rect = self.sprite.rect().extend(64.0);
		let hovered = !ctx.input.is_consumed() && rect.contains(&ctx.input.mouse_pos());
		if !hovered {
			return;
		}

		// Cool down the board when shaking the mouse cursor over the CPU
		if ctx.input.cursor_shaking(Self::SHAKE_THRESHOLD) {
			state.emu.cool_down(4.0);
		}
	}

	pub fn draw(&mut self, ctx: &mut AppContext, state: &State, canvas: CanvasId) {
		let mut opacity = state.emu.heat;
		if opacity > 0.0 && opacity < 1.0 {
			opacity = Easing::OutCubic.apply(opacity);
		}

		self.sprite.opacity = opacity.min(1.0);
		self.sprite.draw(&mut ctx.painter, canvas);
	}
}
