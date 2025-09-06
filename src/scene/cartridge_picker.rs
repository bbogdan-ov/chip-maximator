mod carousel;
mod description;
mod speaker;

use std::time::Duration;

use carousel::Carousel;
use description::Description;
use miniquad::KeyCode;
use speaker::Speaker;

use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	games::GAMES,
	input::InputConsume,
	math::{Color, Point, Rect},
	painter::{self, CanvasId, Sprite},
	state::State,
	util::{Easing, Tweenable},
};

/// Cartridge picker state
#[derive(Default)]
pub struct PickerState {
	/// Equiped cartridge index
	pub equiped_idx: Option<usize>,
	/// Currently dragging cartridge index
	pub dragging_idx: Option<usize>,
	/// Dropped cartridge index
	pub dropped_idx: Option<usize>,
	/// Unequipped cartridge index
	pub unequipped_idx: Option<usize>,

	pub just_grabbed: bool,
	pub just_dropped: bool,
	pub just_equipped: bool,
	pub just_unequipped: bool,
}
impl PickerState {
	pub fn equip(&mut self, idx: usize) {
		self.unequip();
		self.equiped_idx = Some(idx);
		self.just_equipped = true;
	}
	pub fn unequip(&mut self) {
		if self.equiped_idx.is_none() {
			return;
		}

		self.unequipped_idx = self.equiped_idx;
		self.equiped_idx = None;
		self.just_unequipped = true;
	}

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

	spot_tween: Tweenable,
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

			spot_tween: Tweenable::default(),
		}
	}

	pub fn show(&mut self) {
		self.visible = true;
	}
	pub fn hide(&mut self) {
		self.visible = false;
	}

	pub fn update(&mut self, ctx: &mut AppContext, state: &mut State) {
		if !self.visible {
			return;
		}

		self.spot_tween.update(&ctx.time);

		self.end_consume(ctx);

		if ctx.input.key_just_pressed(KeyCode::Escape) {
			self.hide();
			return;
		}

		self.carousel.update(ctx, &mut self.state);

		if self.state.just_grabbed {
			let dur = Duration::from_millis(200);
			self.spot_tween.play(1.0, dur, Easing::InOutSine);
		}

		if self.state.just_dropped
			&& let Some(card_idx) = self.state.dropped_idx
		{
			let dur = Duration::from_millis(400);
			self.spot_tween.play(0.0, dur, Easing::InOutSine);

			if Self::DROP_RECT.contains(&ctx.input.mouse_pos) {
				self.state.equip(card_idx);

				// Load cartridge
				let game = &GAMES[card_idx];
				state.emu.load(game.bytes);
			}
		}

		if self.state.just_unequipped {
			if let Some(prev_idx) = self.state.unequipped_idx {
				// Smoothly move previously equiped cartridge back to the carousel
				self.carousel.cards[prev_idx].play_tween(500);
			}

			if self.state.equiped_idx.is_none() {
				state.emu.program.fill(0);
				state.emu.reset();
			}
		}

		self.speaker.update(ctx, state, &self.state);

		self.state.just_grabbed = false;
		self.state.just_dropped = false;
		self.state.just_equipped = false;
		self.state.just_unequipped = false;

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

		self.draw_bg(ctx, canvas);

		// TEMP: drop rect
		Sprite::new(ctx.painter.white_texture, Self::DROP_RECT.size)
			.with_pos(Self::DROP_RECT.pos)
			.with_fg(Color::gray(0.5))
			.draw(&mut ctx.painter, canvas);

		self.speaker.draw(ctx, canvas, &self.state);
		// self.desc.draw(ctx, canvas);
		self.carousel.draw(ctx, canvas);

		self.begin_consume(ctx);
	}
	fn draw_bg(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		let size = Point::new(CANVAS_WIDTH, CANVAS_HEIGHT);

		let mut pos = Point::default();

		if let Some(idx) = self.state.dragging_idx {
			pos = self.carousel.cards[idx].pos;
		} else if let Some(idx) = self.state.dropped_idx {
			pos = self.carousel.cards[idx].pos;
		}

		ctx.painter.set_uniforms(
			Some(canvas),
			Some((ctx.assets.titles_bg.id, ctx.painter.empty_texture)),
			painter::BatchUniforms {
				flags: painter::BatchFlag::PICKER_BG,
				factor: self.spot_tween.value,
				mouse_pos: pos / size,
				..Default::default()
			},
		);

		ctx.painter
			.push_quad(Point::default(), size, painter::QUAD_FLIPPED_UV, 1.0);
	}

	fn begin_consume(&self, ctx: &mut AppContext) {
		ctx.input.consume(InputConsume::OVERLAY, true);
	}
	fn end_consume(&self, ctx: &mut AppContext) {
		ctx.input.consume(InputConsume::OVERLAY, false);
	}
}
