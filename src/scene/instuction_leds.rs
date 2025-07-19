use crate::{
	app::AppContext,
	painter::{CanvasId, Sprite},
	state::State,
};

/// Front board instuction LEDs
/// Shows current CHIP-8 instuction
#[derive(Default)]
pub struct InstuctionLeds {
	ins: (u8, u8),
}
impl InstuctionLeds {
	const COUNT: usize = 16;

	pub fn draw(&mut self, ctx: &mut AppContext, state: &mut State, canvas: CanvasId) {
		const LEDS_X: f32 = 482.0;
		const LEDS_Y: f32 = 217.0;
		const LEDS_GAP_X: f32 = 15.0;
		const LEDS_GAP_Y: f32 = 16.0;
		const LEDS_IN_ROW: i32 = 8;

		let mut led = Sprite::from(&ctx.assets.led);

		// Update current instuction every 2th frame
		if !state.board.power {
			self.ins = (0, 0);
		} else if ctx.time.elapsed % 2 == 0 {
			self.ins = state.emu.cur_ins;
		}

		for i in 0..Self::COUNT as i32 {
			let mut x = (i % LEDS_IN_ROW) as f32;
			let mut y = (i / LEDS_IN_ROW) as f32;
			x = LEDS_X + LEDS_GAP_X * x;
			y = LEDS_Y + LEDS_GAP_Y * y;

			// Adjust position of some LEDs
			// No `else if` chain is on purpose
			if (4..=7).contains(&i) {
				x += 2.0;
			}
			if i >= 10 {
				x += 1.0;
			}
			if i >= 13 {
				x += 1.0;
			}

			let byte = if i <= LEDS_IN_ROW - 1 {
				self.ins.0 as i32
			} else {
				self.ins.1 as i32
			};

			let mask: i32 = 1 << ((LEDS_IN_ROW - 1) - i % LEDS_IN_ROW);
			let is_active = byte & mask > 0;

			let opacity = &mut state.leds.opacity[i as usize];
			if is_active {
				*opacity = 1.1;
			} else {
				// Smoothly fade out
				*opacity = (*opacity / 1.2).max(0.0);
			}

			led.frame.x = i;
			led.opacity = state.leds.opacity[i as usize].min(1.0);
			led.pos.set(x, y);

			led.draw(&mut ctx.painter, canvas);
		}
	}
}
