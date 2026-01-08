use crate::{
	app::AppContext,
	math::Point,
	painter::{CanvasId, Sprite},
	state::State,
	util::Timer,
};

/// Front board state LEDs
/// Show current emulator states such as errors, power, etc...
pub struct StateLeds {
	show_error_timer: Timer,
	show_used_timer: Timer,
}
impl Default for StateLeds {
	fn default() -> Self {
		Self {
			show_error_timer: Timer::from_millis(600),
			show_used_timer: Timer::from_millis(200),
		}
	}
}
impl StateLeds {
	pub fn update(&mut self, ctx: &mut AppContext, state: &mut State) {
		self.show_error_timer.update(&ctx.time);
		self.show_used_timer.update(&ctx.time);

		if state.emu.error {
			self.show_error_timer.start();
			state.emu.error = false;
		}
		if state.emu.key_checked {
			self.show_used_timer.start();
			state.emu.key_checked = false
		}
	}

	pub fn draw(&mut self, ctx: &mut AppContext, state: &State, canvas: CanvasId) {
		const KEYBOARD_WAIT_POS: Point = Point::new(173.0, 301.0);
		const KEYBOARD_USED_POS: Point = Point::new(187.0, 301.0);
		const POWER_POS: Point = Point::new(495.0, 353.0);
		const ERROR_POS: Point = Point::new(495.0, 363.0);

		if !state.board.power {
			return;
		}

		let mut sprite = Sprite::from(&ctx.assets.state_led);

		if state.emu.wait_for_keypress.is_some() {
			// Keyboard wait LED
			sprite.frame.x = 0;
			sprite.pos = KEYBOARD_WAIT_POS;
			sprite.draw(&mut ctx.painter, canvas);
		}
		if !self.show_used_timer.finished() {
			// Keyboard use LED
			sprite.frame.x = 1;
			sprite.pos = KEYBOARD_USED_POS;
			sprite.draw(&mut ctx.painter, canvas);
		}

		// Power LED
		sprite.pos = POWER_POS;
		sprite.frame.x = 2;
		sprite.draw(&mut ctx.painter, canvas);

		if !self.show_error_timer.finished() && ctx.time.elapsed % 10 < 5 {
			// Error LED
			sprite.pos = ERROR_POS;
			sprite.frame.x = 3;
			sprite.draw(&mut ctx.painter, canvas);
		}
	}
}
