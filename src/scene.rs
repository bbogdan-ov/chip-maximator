mod cartridge_picker;
mod cpu;
mod game_display;
mod instuction_leds;
mod key;
mod keyboard;
mod links;
mod registers_display;
mod reset_button;
mod state_leds;
mod switch;
mod timers;
mod titles_display;
mod valve;

use cartridge_picker::CartridgePicker;
use cpu::Cpu;
use game_display::GameDisplay;
use instuction_leds::InstuctionLeds;
use keyboard::Keyboard;
use links::Links;
use miniquad::{KeyCode, window};
use registers_display::RegistersDisplay;
use reset_button::ResetButton;
use state_leds::StateLeds;
use switch::Switch;
use timers::Timers;
use titles_display::TitlesDisplay;
use valve::Valve;

use crate::{
	audio::Sound,
	input::InputConsume,
	math::{Color, Rect},
	painter::{BlendMode, CanvasId, Icon, IconKind, Merge, Sprite, Text},
	state::{BoardSide, State},
	util::{Anim, AnimRef, AnimWait, Easing, Keyframe, Timeline, TweenPlay},
};

use crate::app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH};

/// Board animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardAnim {
	Front,
	Back,
	Flipping,
	Falling,
}

/// Board timeline action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
	TogglePower,
	SetAnim(BoardAnim),
	Reset,
}

fn new_buzz_sound(ctx: &mut AppContext) -> Sound {
	#[cfg(not(target_arch = "wasm32"))]
	{
		use rodio::source::Source;

		ctx.audio.new_sound(
			rodio::source::TriangleWave::new(200.0)
				.high_pass(500)
				.amplify(0.2),
		)
	}

	#[cfg(target_arch = "wasm32")]
	crate::audio::Sound
}

/// Scene
pub struct Scene {
	front_board: FrontBoard,
	back_board: BackBoard,
	picker: CartridgePicker,

	/// Board power state before flipping
	was_power: bool,
	cur_board_anim: BoardAnim,

	flip_timeline: Timeline<Action>,
	explode_timeline: Timeline<Action>,

	flip_anim: AnimRef,
	fall_anim: AnimRef,
	explosion_anim: AnimRef,

	buzz_sound: Sound,
	whistle_sound: Sound,

	/// Layer with the "normal" blend mode
	normal_layer: CanvasId,
	/// Layer with the "addition" blend mode
	add_layer: CanvasId,
}
impl Scene {
	pub fn new(ctx: &mut AppContext, state: &State) -> Self {
		let front_board = FrontBoard::new(ctx);

		let flip_anim = Anim::new(8, 0..ctx.assets.board_flip.frames.x).into_ref();
		let fall_anim = Anim::new(8, 0..ctx.assets.board_fall.frames.x).into_ref();
		let explosion_anim = Anim::new(16, 0..ctx.assets.explosion.frames.x).into_ref();

		let flip_timeline = Timeline::new([
			Keyframe::action(Action::TogglePower),
			Keyframe::tween(
				state.valve.angle.clone(),
				TweenPlay::new(0.0, 300, Easing::InOutSine),
			),
			Keyframe::action(Action::SetAnim(BoardAnim::Front)),
			Keyframe::group([
				Keyframe::sound(ctx.assets.swipe_sound),
				Keyframe::anim(flip_anim.clone(), AnimWait::Finish),
				Keyframe::action(Action::SetAnim(BoardAnim::Flipping)),
			]),
			Keyframe::action(Action::SetAnim(BoardAnim::Back)),
			Keyframe::delay(1000 / 8),
		]);

		let explode_timeline = Timeline::new([
			Keyframe::sound(ctx.assets.explosion_sound),
			Keyframe::anim(explosion_anim.clone(), AnimWait::Frame(3)),
			Keyframe::action(Action::Reset),
			Keyframe::action(Action::SetAnim(BoardAnim::Falling)),
			Keyframe::delay(1500),
			Keyframe::sound(ctx.assets.fall_sound),
			Keyframe::delay(500),
			Keyframe::anim(fall_anim.clone(), AnimWait::Finish),
			Keyframe::action(Action::SetAnim(BoardAnim::Front)),
		]);

		let whistle_sound = ctx
			.audio
			.new_sound_from_vorbis(ctx.assets.whistle_sound, true);
		whistle_sound.set_volume(0.0);
		whistle_sound.play();

		Self {
			front_board,
			back_board: BackBoard::new(ctx),
			picker: CartridgePicker::default(),

			was_power: state.board.power,
			cur_board_anim: match state.board.side {
				BoardSide::Front => BoardAnim::Front,
				BoardSide::Back => BoardAnim::Back,
			},

			flip_timeline,
			explode_timeline,

			flip_anim,
			fall_anim,
			explosion_anim,

			buzz_sound: new_buzz_sound(ctx),
			whistle_sound,

			normal_layer: ctx.painter.context.new_canvas(
				(CANVAS_WIDTH, CANVAS_HEIGHT),
				Color::TRANSPARENT,
				Default::default(),
			),
			add_layer: ctx.painter.context.new_canvas(
				(CANVAS_WIDTH, CANVAS_HEIGHT),
				Color::TRANSPARENT,
				Default::default(),
			),
		}
	}

	fn flip(&mut self, ctx: &mut AppContext, state: &State) {
		// Don't flip the board if CPU is hot
		if state.emu.is_hot() {
			ctx.tooltip.set_error(b"CPU is too hot!");
			return;
		}

		match self.cur_board_anim {
			BoardAnim::Front => self.flip_timeline.play(false),
			BoardAnim::Back => self.flip_timeline.play(true),
			BoardAnim::Flipping => (),
			BoardAnim::Falling => (),
		}
	}
	fn explode(&mut self) {
		self.explode_timeline.play(false);
	}

	pub fn update(&mut self, ctx: &mut AppContext, state: &mut State) {
		self.update_anims(ctx);
		self.update_timelines(ctx, state);
		self.update_boards(ctx, state);
		self.update_emu(state);
		self.update_heat(state);

		if cfg!(debug_assertions) && ctx.input.key_just_pressed(KeyCode::Enter) {
			self.explode();
		}

		// Consume input when any board anim is playing
		ctx.input.consume(
			InputConsume::BOARD_ANIM,
			self.flip_timeline.playing || self.explode_timeline.playing,
		);
	}
	fn update_boards(&mut self, ctx: &mut AppContext, state: &mut State) {
		let back_factor = match self.cur_board_anim {
			BoardAnim::Front => 0.0,
			BoardAnim::Back => 1.0,
			BoardAnim::Flipping => self.flip_anim.borrow().progress(),
			BoardAnim::Falling => 0.0,
		};

		self.front_board.update(ctx, state, 1.0 - back_factor);
		self.back_board.update(ctx, back_factor);
	}
	fn update_emu(&mut self, state: &mut State) {
		// Update emulator
		if state.board.power {
			state.emu.update();
		}

		// Update buzz sound
		let play = state.emu.sound_timer > 0 && state.board.power;
		self.buzz_sound.set_playing(play);
	}
	fn update_heat(&mut self, state: &mut State) {
		state.emu.cool_down(1.0);

		if state.emu.is_critical_heat() {
			self.explode();
		}

		// Update whistle sound
		let volume = ((state.emu.heat - 0.6) / 0.3).clamp(0.0, 1.0);
		self.whistle_sound.set_volume(volume);
	}

	fn update_anims(&mut self, ctx: &AppContext) {
		self.flip_anim.borrow_mut().update(&ctx.time);
		self.fall_anim.borrow_mut().update(&ctx.time);
		self.explosion_anim.borrow_mut().update(&ctx.time);
	}
	fn update_timelines(&mut self, ctx: &mut AppContext, state: &mut State) {
		self.flip_timeline.update(&ctx.time);
		self.explode_timeline.update(&ctx.time);

		while let Some(action) = self.flip_timeline.next(ctx) {
			self.handle_action(ctx, state, action, self.flip_timeline.reversed);
		}
		while let Some(action) = self.explode_timeline.next(ctx) {
			self.handle_action(ctx, state, action, self.explode_timeline.reversed);
		}
	}

	fn handle_action(
		&mut self,
		ctx: &mut AppContext,
		state: &mut State,
		action: Action,
		reversed: bool,
	) {
		match action {
			Action::TogglePower => {
				if reversed {
					state.board.switch_power(ctx, self.was_power);
				} else {
					self.was_power = state.board.power;
					state.board.switch_power(ctx, false);
				}
			}
			Action::SetAnim(anim) => self.cur_board_anim = anim,
			Action::Reset => state.reset(),
		}
	}

	/// Draw things onto the canvas
	pub fn draw(&mut self, ctx: &mut AppContext, state: &mut State, canvas: CanvasId) {
		self.offscreen_draw(ctx, state);
		self.canvas_draw(ctx, state, canvas);
	}
	/// Draw things right onto the screen, skipping drawing onto a canvas
	pub fn screen_draw(&mut self, ctx: &mut AppContext) {
		self.draw_explosion(ctx);
	}

	fn offscreen_draw(&mut self, ctx: &mut AppContext, state: &mut State) {
		match self.cur_board_anim {
			// Front board
			BoardAnim::Front => {
				self.front_board.draw(ctx, state, self.normal_layer);
				self.front_board.draw_displays(ctx, state, self.add_layer);
			}
			// Back board
			BoardAnim::Back => {
				let frame = ctx.assets.titles_display_uv.frames.x - 1;

				self.back_board.draw(ctx, self.normal_layer);
				self.back_board.draw_displays(ctx, self.add_layer, frame);
			}
			// Flipping board
			BoardAnim::Flipping => self.draw_flipping(ctx),
			// Falling board
			BoardAnim::Falling => self.draw_falling(ctx),
		}
	}
	fn draw_flipping(&mut self, ctx: &mut AppContext) {
		let flip_anim = self.flip_anim.borrow();

		Sprite::from(&ctx.assets.board_flip)
			.with_anim(&flip_anim)
			.draw(&mut ctx.painter, self.normal_layer);

		let flip_frames = ctx.assets.board_flip.frames.x;
		let uv_frames = ctx.assets.titles_display_uv.frames.x;
		let frame = flip_anim.frame - (flip_frames - uv_frames) - 1;

		if frame >= 0 {
			self.back_board.draw_displays(ctx, self.add_layer, frame);
		}
	}
	fn draw_falling(&mut self, ctx: &mut AppContext) {
		let fall_anim = self.fall_anim.borrow();

		if fall_anim.playing {
			Sprite::from(&ctx.assets.board_fall)
				.with_anim(&fall_anim)
				.draw(&mut ctx.painter, self.normal_layer);
		}
	}

	fn canvas_draw(&mut self, ctx: &mut AppContext, state: &mut State, canvas: CanvasId) {
		// Make the entire board jitter when it is too hot
		let mut x = 0.0;
		if state.emu.heat > 0.6 {
			let f = (state.emu.heat - 0.6) / 0.4;
			x = (ctx.time.elapsed as f32 * 2.0).sin() * 2.0 * f;
		}

		// Merge board and its displays
		Merge::new(
			ctx.painter.canvas(self.normal_layer).texture,
			ctx.painter.canvas(self.add_layer).texture,
			BlendMode::Add,
		)
		.with_factor(0.6)
		.with_pos((x, 0.0))
		.draw(&mut ctx.painter, canvas);

		self.draw_flip_trigger(ctx, state, canvas);
		self.draw_tooltip(ctx, canvas);

		// self.picker.draw(ctx, canvas);
	}

	fn draw_tooltip(&self, ctx: &mut AppContext, canvas: CanvasId) {
		let timer = &ctx.tooltip.error_timer;
		let error_visible = !timer.finished() && timer.left().as_millis() % 200 < 100;
		let visible = ctx.tooltip.is_tooltip_visible;

		let msg = if error_visible {
			&ctx.tooltip.error
		} else {
			&ctx.tooltip.tooltip
		};

		if visible || error_visible {
			Text::new(&ctx.assets.ibm_font)
				.with_pos((90.0, CANVAS_HEIGHT - 72.0))
				.draw_line(&mut ctx.painter, canvas, msg);
		}
	}
	fn draw_flip_trigger(&mut self, ctx: &mut AppContext, state: &mut State, canvas: CanvasId) {
		const TRIGGER_W: f32 = 90.0;
		const TRIGGER_H: f32 = 300.0;

		const RIGHT_FLIP_TRIGGER: Rect = Rect::new_xywh(
			CANVAS_WIDTH - TRIGGER_W,
			CANVAS_HEIGHT / 2.0 - TRIGGER_H / 2.0,
			TRIGGER_W,
			TRIGGER_H,
		);
		const LEFT_FLIP_TRIGGER: Rect = Rect::new_xywh(
			0.0,
			CANVAS_HEIGHT / 2.0 - TRIGGER_H / 2.0,
			TRIGGER_W,
			TRIGGER_H,
		);

		let rect = match self.cur_board_anim {
			BoardAnim::Front => RIGHT_FLIP_TRIGGER,
			BoardAnim::Back => LEFT_FLIP_TRIGGER,
			_ => return,
		};

		if rect.is_hover(&mut ctx.input) {
			ctx.tooltip.set(b"Flip the board");

			// Draw flip icon
			let flip = matches!(self.cur_board_anim, BoardAnim::Back);
			Icon::new(&ctx.assets, IconKind::Flip)
				.with_pos(rect.center())
				.with_flip((flip, false))
				.draw(ctx, canvas);

			if ctx.input.left_just_pressed() {
				self.flip(ctx, state);
			}
		}
	}
	fn draw_explosion(&mut self, ctx: &mut AppContext) {
		let explosion_anim = self.explosion_anim.borrow();
		if !explosion_anim.playing {
			return;
		}

		let texture = &ctx.assets.explosion;

		let (sw, sh) = window::screen_size();
		let size = texture.size.x;
		let scale = sh / size * 2.0;
		let h = size * scale / 2.0;
		Sprite::from(texture)
			.with_flip((false, true))
			.with_anim(&explosion_anim)
			.with_pos((sw / 2.0 - h + 100.0, sh / 2.0 - h + sh * 0.3))
			.with_scale(scale)
			.draw_screen(&mut ctx.painter);
	}
}

/// Front board
struct FrontBoard {
	game_display: GameDisplay,
	registers_display: RegistersDisplay,

	keyboard: Keyboard,
	valve: Valve,
	ins_leds: InstuctionLeds,
	switch: Switch,
	timers: Timers,
	state_leds: StateLeds,
	reset_button: ResetButton,
	cpu: Cpu,
}
impl FrontBoard {
	fn new(ctx: &mut AppContext) -> Self {
		Self {
			game_display: GameDisplay::new(ctx),
			registers_display: RegistersDisplay::new(ctx),

			valve: Valve::new(ctx),
			ins_leds: InstuctionLeds::default(),
			state_leds: StateLeds::default(),
			keyboard: Keyboard,
			switch: Switch,
			timers: Timers,
			reset_button: ResetButton,
			cpu: Cpu::new(ctx),
		}
	}

	fn update(&mut self, ctx: &mut AppContext, state: &mut State, factor: f32) {
		if factor == 0.0 {
			return;
		}

		self.cpu.update(ctx, state);
		self.game_display.update(ctx, state);
		self.state_leds.update(ctx, state);
		self.valve.update(ctx, state);
	}

	fn draw(&mut self, ctx: &mut AppContext, state: &mut State, canvas: CanvasId) {
		// Draw board
		Sprite::from(&ctx.assets.front_board).draw(&mut ctx.painter, canvas);

		// Draw components
		self.cpu.draw(ctx, state, canvas);
		self.ins_leds.draw(ctx, state, canvas);
		self.keyboard.draw(ctx, state, canvas);
		self.reset_button.draw(ctx, state, canvas);
		self.state_leds.draw(ctx, state, canvas);
		self.switch.draw(ctx, state, canvas);
		self.timers.draw(ctx, state, canvas);
		self.valve.draw(ctx, state, canvas);

		Sprite::from(&ctx.assets.slot)
			.with_frame((1, 0))
			.with_pos((534.0, 93.0))
			.draw(&mut ctx.painter, canvas);
	}
	fn draw_displays(&mut self, ctx: &mut AppContext, state: &mut State, canvas: CanvasId) {
		if !state.board.power {
			return;
		}

		// Draw displays textures
		self.game_display.offscreen_draw(ctx, state);
		self.registers_display.offscreen_draw(ctx, state);

		// Draw game display
		Sprite::from(ctx.painter.canvas(self.game_display.canvas))
			.with_uv(ctx.assets.game_display_uv.id)
			.draw(&mut ctx.painter, canvas);

		// Draw registers display
		Sprite::from(ctx.painter.canvas(self.registers_display.canvas))
			.with_uv(ctx.assets.registers_display_uv.id)
			.draw(&mut ctx.painter, canvas);
	}
}

/// Back board
struct BackBoard {
	links: Links,
	titles_display: TitlesDisplay,

	anim: Anim,

	fan_sound: Sound,
}
impl BackBoard {
	fn new(ctx: &mut AppContext) -> Self {
		let frames = ctx.assets.back_board.frames.x;
		let anim = Anim::new(8, 0..frames).with_looped().with_playing();

		let fan_sound = ctx.audio.new_sound_from_vorbis(ctx.assets.fan_sound, true);
		fan_sound.set_volume(0.0);
		fan_sound.play();

		Self {
			links: Links,
			titles_display: TitlesDisplay::new(ctx),

			anim,

			fan_sound,
		}
	}

	fn update(&mut self, ctx: &mut AppContext, factor: f32) {
		// Apply some math magic to make the sound appear a bit later and disappear a bit earlier
		let volume = (factor * 2.0 - 0.5).clamp(0.0, 1.0);
		self.fan_sound.set_volume(volume);

		if factor == 0.0 {
			self.anim.frame = 0;
			return;
		}

		self.anim.update(&ctx.time);

		self.titles_display.update(ctx);
	}

	fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		// Draw board
		Sprite::from(&ctx.assets.back_board)
			.with_anim(&self.anim)
			.draw(&mut ctx.painter, canvas);

		// Draw components
		self.links.draw(ctx, canvas);
	}
	/// Draw displays onto the canvas
	/// `frame` - controls which frame for each display UV will be picked
	fn draw_displays(&mut self, ctx: &mut AppContext, canvas: CanvasId, frame: i32) {
		// Draw displays textures
		self.titles_display.offscreen_draw(ctx);

		// Draw titles display
		let titles_uv = &ctx.assets.titles_display_uv;
		Sprite::from(ctx.painter.canvas(self.titles_display.canvas))
			.with_uv(titles_uv.id)
			.with_frames_count(titles_uv.frames)
			.with_frame((frame, 0))
			.draw(&mut ctx.painter, canvas);
	}
}
