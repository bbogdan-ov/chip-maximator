use crate::{
	app::AppContext,
	math::{FloatMath, Point},
	painter::{CanvasId, Sprite},
	state::State,
};

/// Front board timers indicator
/// Shows current value of delay timer and sound timer clamped to `0..=128`
#[derive(Default)]
pub struct Timers;
impl Timers {
	fn calc_crop(value: u8) -> f32 {
		/// Number of divisions in indicator
		const DIVS: f32 = 12.0;
		const THRESHOLD: f32 = 10.0;
		const MAX: f32 = 128.0;

		let crop = ((value as f32 + THRESHOLD) / MAX).snap_floor(1.0 / DIVS) / 1.38 + 0.08;
		crop.clamp(0.0, 1.0)
	}

	pub fn draw(&self, ctx: &mut AppContext, state: &State, canvas: CanvasId) {
		const POS: Point = Point::new(533.0, 355.0);

		if !state.board.power {
			return;
		}

		let mut sprite = Sprite::from(&ctx.assets.timers).with_pos(POS);

		// Delay timer
		sprite.crop.y = Self::calc_crop(state.emu.delay_timer);
		sprite.draw(&mut ctx.painter, canvas);

		// Sound timer
		sprite.pos.x += sprite.size.x;
		sprite.crop.y = Self::calc_crop(state.emu.sound_timer);
		sprite.frame.x = 1;
		sprite.draw(&mut ctx.painter, canvas);
	}
}
