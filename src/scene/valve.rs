use crate::{
	app::AppContext,
	input::InputConsume,
	math::Point,
	painter::{CanvasId, Sprite},
	state::State,
};

/// Front board valve
/// Controls the CHIP-8 exection speed
pub struct Valve {
	sprite: Sprite,

	start_mouse_dir: Point,
	start_angle: f32,
	last_angle: f32,

	// These two are named badly but i don't care
	sounds_played: u8,
	sound_degrees: f32,
}
impl Valve {
	const STRENGH: f32 = 0.002;

	pub fn new(ctx: &AppContext) -> Self {
		Self {
			sprite: Sprite::from(&ctx.assets.valve).with_pos((520.0, 505.0)),

			start_mouse_dir: Point::default(),
			start_angle: 0.0,
			last_angle: 0.0,

			sounds_played: 0,
			sound_degrees: 0.0,
		}
	}

	fn rotate_to(&mut self, state: &mut State, pos: Point) {
		let dir = pos - self.sprite.rect().center();
		let mouse_angle = self.start_mouse_dir.angle(dir).to_degrees();

		state.valve.angle.borrow_mut().value = (self.start_angle + mouse_angle) % 360.0;
	}

	pub fn update(&mut self, ctx: &mut AppContext, state: &mut State) {
		state.valve.angle.borrow_mut().update(&ctx.time);

		self.update_rotation(ctx, state);
		self.update_change(ctx, state);

		self.last_angle = state.valve.angle.borrow().value;
	}
	fn update_change(&mut self, ctx: &mut AppContext, state: &mut State) {
		let angle = state.valve.angle.borrow().value;

		// Calculate rotation difference from the last frame
		let mut diff = angle - self.last_angle;
		if angle < 90.0 && self.last_angle > 270.0 {
			diff = 360.0 - self.last_angle + angle;
		} else if angle > 270.0 && self.last_angle < 90.0 {
			diff = -(360.0 - angle + self.last_angle);
		}

		// Play rotation sound
		self.sound_degrees += diff.abs();
		if self.sounds_played > 0 {
			self.sounds_played -= 1;
		}
		if self.sounds_played < 2
			&& self.sound_degrees >= 360.0 / 5.0 / self.sprite.frames_count.x as f32
		{
			ctx.audio.play(ctx.assets.rotation_sound);
			self.sounds_played += 3;
			self.sound_degrees = 0.0;
		}

		// Increase emulator speed
		if state.valve.is_rotating && state.board.power {
			state.emu.inc_speed(diff * Self::STRENGH);
		}
	}
	fn update_rotation(&mut self, ctx: &mut AppContext, state: &mut State) {
		let hovered = self.sprite.is_hover(&mut ctx.input);

		if hovered {
			ctx.tooltip.set(b"Valve");
		}

		if hovered && ctx.input.left_just_pressed() {
			// Grab valve
			state.valve.is_rotating = true;

			let angle = state.valve.angle.borrow().value;

			self.start_mouse_dir = ctx.input.mouse_pos - self.sprite.rect().center();
			self.start_angle = angle;
		} else if ctx.input.mouse_just_released {
			// Release valve
			state.valve.is_rotating = false;
		}

		// Rotate valve
		if state.valve.is_rotating {
			self.rotate_to(state, ctx.input.mouse_pos);
		}

		ctx.input
			.consume(InputConsume::VALVE, state.valve.is_rotating);
	}

	pub fn draw(&mut self, ctx: &mut AppContext, state: &State, canvas: CanvasId) {
		let angle = state.valve.angle.borrow().value;

		// Set rotation frame
		let frames = self.sprite.frames_count.x;
		self.sprite.frame.x = (angle / frames as f32 / 2.0) as i32 % frames;

		self.sprite.draw(&mut ctx.painter, canvas);
	}
}
