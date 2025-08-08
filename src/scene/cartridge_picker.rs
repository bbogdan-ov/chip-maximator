mod carousel;
mod description;
mod speaker;

use carousel::Carousel;
use description::Description;
use miniquad::KeyCode;
use speaker::Speaker;

use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	games::GAMES,
	input::InputConsume,
	math::{Color, Rect},
	painter::{CanvasId, Sprite},
	state::State,
};

/// Cartridge picker state
pub struct PickerState {
	/// Equiped cartridge index
	pub equiped_idx: Option<usize>,
	/// Currently dragging cartridge index
	pub dragging_idx: Option<usize>,
	/// Dropped cartridge index
	pub dropped_idx: Option<usize>,
}
impl Default for PickerState {
	fn default() -> Self {
		Self {
			equiped_idx: None,
			dragging_idx: None,
			dropped_idx: None,
		}
	}
}
impl PickerState {
	pub fn is_dragging(&self, idx: usize) -> bool {
		self.dragging_idx.is_some_and(|i| i == idx)
	}
	pub fn is_dragging_any(&self) -> bool {
		self.dragging_idx.is_some()
	}
}

/// Cartridge picker
pub struct CartridgePicker {
	state: PickerState,
	visible: bool,

	carousel: Carousel,
	speaker: Speaker,
	desc: Description,
}
impl CartridgePicker {
	const DROP_RECT: Rect = Rect::new_xywh(180.0, 80.0, 180.0, 180.0);

	pub fn new(ctx: &mut AppContext) -> Self {
		Self {
			state: PickerState::default(),
			visible: false,

			carousel: Carousel::new(ctx),
			speaker: Speaker::new(),
			desc: Description::new(ctx),
		}
	}

	pub fn show(&mut self) {
		self.visible = true;
	}
	pub fn hide(&mut self) {
		self.visible = false;
	}

	fn equip(&mut self, state: &mut State, idx: usize) {
		if let Some(prev_idx) = self.state.equiped_idx {
			// Smoothly move previously equiped cartridge back to the carousel
			self.carousel.cards[prev_idx].play_tween(500);
		}

		self.state.equiped_idx = Some(idx);

		let game = &GAMES[idx];
		state.emu.load(game.bytes);
	}

	pub fn update(&mut self, ctx: &mut AppContext, state: &mut State) {
		if !self.visible {
			return;
		}

		self.end_consume(ctx);

		if ctx.input.key_just_pressed(KeyCode::Escape) {
			self.hide();
			return;
		}

		self.carousel.update(ctx, &mut self.state);
		self.speaker.update(ctx, state);

		if let Some(card_idx) = self.state.dropped_idx.take() {
			if Self::DROP_RECT.contains(&ctx.input.mouse_pos) {
				self.equip(state, card_idx);
			}
		}

		self.begin_consume(ctx);
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		if !self.visible {
			return;
		}

		self.desc.offscreen_draw(ctx, &self.state);
	}
	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		if !self.visible {
			return;
		}

		self.end_consume(ctx);

		// Draw darken rect
		Sprite::new(ctx.painter.white_texture, (CANVAS_WIDTH, CANVAS_HEIGHT))
			.with_fg((0.0, 0.0, 0.0))
			.draw(&mut ctx.painter, canvas);

		// TEMP: drop rect
		Sprite::new(ctx.painter.white_texture, Self::DROP_RECT.size)
			.with_pos(Self::DROP_RECT.pos)
			.with_fg(Color::gray(0.5))
			.draw(&mut ctx.painter, canvas);

		self.speaker.draw(ctx, canvas);
		self.desc.draw(ctx, canvas);
		self.carousel.draw(ctx, canvas);

		self.begin_consume(ctx);
	}

	fn begin_consume(&self, ctx: &mut AppContext) {
		ctx.input.consume(InputConsume::OVERLAY, true);
	}
	fn end_consume(&self, ctx: &mut AppContext) {
		ctx.input.consume(InputConsume::OVERLAY, false);
	}
}
