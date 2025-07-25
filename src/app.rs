use std::{path::Path, time::Duration};

use miniquad::{EventHandler, KeyCode, KeyMods, MouseButton, window};

use crate::{
	assets::Assets,
	audio::Audio,
	cli::Cli,
	emu::Emu,
	input::Input,
	math::{Color, Point},
	native::NativeInstant,
	painter::{CanvasId, Painter, Sprite},
	scene::Scene,
	state::State,
	tooltip::Tooltip,
	util::Anim,
};

pub const CANVAS_WIDTH: f32 = 700.0;
pub const CANVAS_HEIGHT: f32 = 700.0;

const DEFAULT_ROM: &[u8] = include_bytes!("../roms/outlaw.ch8");

/// Time
pub struct Time {
	last_time: NativeInstant,
	/// Time elapsed from the last frame
	pub delta: Duration,
	/// Number of frames elapsed from the app start
	pub elapsed: u32,
}
impl Default for Time {
	fn default() -> Self {
		Self {
			last_time: NativeInstant::now(),
			delta: Duration::default(),
			elapsed: 0,
		}
	}
}
impl Time {
	fn update(&mut self) {
		self.elapsed = self.elapsed.wrapping_add(1);

		self.delta = self.last_time.elapsed();
		self.last_time = NativeInstant::now();
	}

	pub fn fps(&self) -> u32 {
		(self.delta.as_millis() as f32 / (1000.0 / 60.0) * 60.0) as u32
	}
}

/// App context
pub struct AppContext {
	pub assets: Assets,
	pub input: Input,
	pub time: Time,
	pub tooltip: Tooltip,

	pub painter: Painter,
	pub audio: Audio,

	pub icons_anim: Anim,
}

#[allow(clippy::unused_io_amount)]
#[cfg(not(target_arch = "wasm32"))]
fn read_rom(path: impl AsRef<Path>) -> Option<[u8; Emu::PROGRAM_SIZE]> {
	// TODO: notify users about ROM reading errors

	use crate::emu::Emu;
	use std::io::Read;

	let Ok(mut file) = std::fs::File::open(path) else {
		return None;
	};

	let mut buf = [0_u8; Emu::PROGRAM_SIZE];
	if file.read(&mut buf).is_ok() {
		Some(buf)
	} else {
		None
	}
}

/// App
pub struct App {
	pub scene: Scene,
	pub context: AppContext,
	pub state: State,

	/// Canvas where everything will be drawn later
	pub canvas: CanvasId,
	pub canvas_offset: Point,
	pub canvas_scale: f32,
}
impl App {
	pub fn new(cli: Cli) -> Self {
		// TODO: put code in order

		let mut painter = Painter::new().unwrap_or_else(|e| {
			panic!("failed to initialize painter: {e}");
		});

		let audio = Audio::new(cli.muted);

		let canvas = painter.context.new_canvas(
			(CANVAS_WIDTH, CANVAS_HEIGHT),
			Color::BLACK,
			Default::default(),
		);

		let mut context = AppContext {
			assets: Assets::new(&mut painter),
			input: Input::default(),
			time: Time::default(),
			tooltip: Tooltip::default(),

			icons_anim: Anim::new(8, 0..4).with_looped().with_playing(),

			painter,
			audio,
		};

		let mut state = State::new();

		if cfg!(debug_assertions) {
			state.board.power = true;
		}

		#[cfg(target_arch = "wasm32")]
		state.emu.load(DEFAULT_ROM);

		#[cfg(not(target_arch = "wasm32"))]
		match cli.rom_path.and_then(read_rom) {
			Some(bytes) => state.emu.load(&bytes),
			None => state.emu.load(DEFAULT_ROM),
		}

		Self {
			scene: Scene::new(&mut context, &state),
			context,
			state,

			canvas,
			canvas_offset: Point::default(),
			canvas_scale: 1.0,
		}
	}

	fn update_canvas_scaling(&mut self) {
		let (sw, sh) = window::screen_size();
		let canvas = self.context.painter.canvas(self.canvas);

		// Scale canvas
		let min_size = sw.min(sh);
		if canvas.size().y * 0.9 > min_size {
			self.canvas_scale = min_size / canvas.size().y;
		} else {
			self.canvas_scale = (min_size / canvas.size().y).floor().max(1.0);
		}

		// Place canvas at the screen center
		self.canvas_offset.x = ((sw - canvas.size().x * self.canvas_scale) / 2.0).floor();
		self.canvas_offset.y = ((sh - canvas.size().y * self.canvas_scale) / 2.0).floor();
	}
}
impl EventHandler for App {
	fn update(&mut self) {
		let ctx = &mut self.context;

		ctx.time.update();
		ctx.tooltip.update(&ctx.time);

		ctx.icons_anim.update(&ctx.time);

		self.scene.update(ctx, &mut self.state);
	}
	fn draw(&mut self) {
		self.update_canvas_scaling();

		let ctx = &mut self.context;
		ctx.painter.begin_frame();

		// Draw scene
		self.scene.draw(ctx, &mut self.state, self.canvas);

		// Draw canvas onto the screen
		Sprite::from(ctx.painter.canvas(self.canvas))
			.with_flip((false, true))
			.with_pos(self.canvas_offset)
			.with_scale(self.canvas_scale)
			.draw_screen(&mut ctx.painter);

		// Draw ontop of the canvas
		self.scene.screen_draw(ctx);

		ctx.painter.commit_frame();

		ctx.input.update_after();
	}

	fn window_minimized_event(&mut self) {
		let ctx = &mut self.context;

		ctx.input.mouse_is_pressed = false;
		ctx.input.mouse_just_pressed = false;
		ctx.input.mouse_just_released = false;
		ctx.input.key_just_pressed = false;
		ctx.input.keys_pressed.clear();

		self.state.emu.pressed_keys.fill(false);
	}
	fn mouse_motion_event(&mut self, mut x: f32, mut y: f32) {
		x = (x - self.canvas_offset.x) / self.canvas_scale;
		y = (y - self.canvas_offset.y) / self.canvas_scale;

		self.context.input.real_mouse_pos.set(x, y);
		self.context.input.mouse_pos.set(x, y);
	}
	fn mouse_button_down_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
		self.context.input.mouse_button = button;
		self.context.input.mouse_is_pressed = true;
		self.context.input.mouse_just_pressed = true;
	}
	fn mouse_button_up_event(&mut self, _button: MouseButton, _x: f32, _y: f32) {
		self.context.input.mouse_is_pressed = false;
		self.context.input.mouse_just_released = true;
	}
	fn key_down_event(&mut self, key: KeyCode, _mods: KeyMods, repeat: bool) {
		// Ingore repeating key presses
		if repeat {
			return;
		}

		let ctx = &mut self.context;

		ctx.input.keys_pressed.insert(key);
		ctx.input.key_just_pressed = true;
	}
	fn key_up_event(&mut self, key: KeyCode, _keymods: KeyMods) {
		let ctx = &mut self.context;

		ctx.input.keys_pressed.remove(&key);
		ctx.input.keys_just_released.insert(key);
	}
}
