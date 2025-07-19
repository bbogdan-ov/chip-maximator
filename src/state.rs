use crate::{
	app::AppContext,
	emu::Emu,
	util::{Tweenable, TweenableRef},
};

/// Board side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardSide {
	Front,
	Back,
}

/// State
pub struct State {
	pub emu: Emu,
	pub board: BoardState,
	pub valve: ValveState,
	pub leds: InstuctionLedsState,
}
impl State {
	pub fn new() -> Self {
		Self {
			emu: Emu::default(),
			board: BoardState::default(),
			valve: ValveState::default(),
			leds: InstuctionLedsState::default(),
		}
	}

	pub fn reset(&mut self) {
		// A cool sword
		// ===):::::::::::::::>

		self.emu = Emu {
			program: self.emu.program,
			..Default::default()
		};
		self.emu.setup();

		self.board = Default::default();
		self.leds = Default::default();

		self.valve.angle.borrow_mut().value = 0.0;
	}
}

/// Board state
pub struct BoardState {
	/// Whether the board is currently turned on
	pub power: bool,
	pub side: BoardSide,
}
impl Default for BoardState {
	fn default() -> Self {
		Self {
			power: false,
			side: BoardSide::Front,
		}
	}
}
impl BoardState {
	/// Set `power` and play the switch sound
	pub fn switch_power(&mut self, ctx: &mut AppContext, on: bool) {
		// Play switch sound
		if self.power != on {
			ctx.audio.play(ctx.assets.switch_toggle_sound);
		}

		self.power = on;
	}
	/// Toggle `power` and play the switch sound
	pub fn toggle_power(&mut self, ctx: &mut AppContext) {
		self.switch_power(ctx, !self.power);
	}
}

/// Valve state
pub struct ValveState {
	/// Valve rotation angle in degrees
	pub angle: TweenableRef,
	/// Whether the valve is currently rotating with the mouse
	pub is_rotating: bool,
}
impl Default for ValveState {
	fn default() -> Self {
		Self {
			angle: Tweenable::default().into_ref(),
			is_rotating: false,
		}
	}
}

/// Instuction LEDs state
#[derive(Default)]
pub struct InstuctionLedsState {
	// Opacity of each LED
	pub opacity: [f32; Self::COUNT],
}
impl InstuctionLedsState {
	pub const COUNT: usize = 16;
}
