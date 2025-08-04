use std::{
	cmp::Ordering,
	f32::consts::{PI, TAU},
	time::Duration,
};

use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	games::{GAMES, GameInfo},
	math::{Color, FloatMath, Lerp, Point},
	painter::{CanvasId, Sprite, Text},
	util::{Anim, Easing, Tweenable},
};

/// Carousel state
pub struct CarouselState {
	/// Currently dragging cartridge index
	pub dragging_idx: Option<usize>,
	pub just_dragged: bool,
}
impl Default for CarouselState {
	fn default() -> Self {
		Self {
			dragging_idx: None,
			just_dragged: false,
		}
	}
}
impl CarouselState {
	pub fn is_dragging(&self, idx: usize) -> bool {
		self.dragging_idx.is_some_and(|i| i == idx)
	}
	pub fn is_dragging_any(&self) -> bool {
		self.dragging_idx.is_some()
	}
}

/// Cartridge card sprite
struct Card {
	info: &'static GameInfo,
	sprite: Sprite,

	pos: Point,
	/// Position of the previous frame
	prev_pos: Point,
	pos_z: f32,
	pos_tween: Tweenable,
	/// Angle velocity
	angle_velocity: f32,

	is_trying_to_drag: bool,

	anim: Anim,
}
impl Card {
	fn new(ctx: &mut AppContext, info: &'static GameInfo) -> Self {
		Self {
			info,
			sprite: Sprite::from(&ctx.assets.cartridge).with_origin((0.5, 0.1)),

			pos: Point::default(),
			prev_pos: Point::default(),
			pos_z: 0.0,
			pos_tween: Tweenable::new(1.0),
			angle_velocity: 0.0,

			is_trying_to_drag: false,

			anim: Anim::new(8, 8..16).with_looped().with_playing(),
		}
	}

	fn start_drag(&mut self, state: &mut CarouselState, idx: usize) {
		if state.is_dragging_any() {
			return;
		}

		let dur = Duration::from_millis(300);
		self.pos_tween.play_from(0.0, 1.0, dur, Easing::Linear);

		state.dragging_idx = Some(idx);
		state.just_dragged = true;
		self.is_trying_to_drag = false;
	}
	fn end_drag(&mut self, state: &mut CarouselState) {
		let dur = Duration::from_millis(500);
		self.pos_tween.play_from(0.0, 1.0, dur, Easing::InOutSine);

		state.dragging_idx = None;
		self.is_trying_to_drag = false;
	}

	fn lerp_to(&mut self, pos: Point, pos_z: f32) {
		let t = self.pos_tween.value;
		self.prev_pos = self.pos;
		self.pos = self.pos.lerp(pos, t);
		self.pos_z = self.pos_z.lerp(pos_z, t);
	}

	fn update(
		&mut self,
		ctx: &mut AppContext,
		state: &mut CarouselState,
		idx: usize,
		target_pos: Point,
		target_z: f32,
	) {
		self.pos_tween.update(&ctx.time);

		if state.is_dragging(idx) {
			// Drag
			let pos = ctx.input.mouse_pos;
			self.lerp_to((pos.x, pos.y + self.sprite.size.y / 3.0).into(), 0.0);

			if ctx.input.left_just_released() {
				self.end_drag(state);
			}
		} else {
			self.lerp_to(target_pos, target_z);

			if self.is_trying_to_drag {
				// Add subtle damping
				self.pos.x += ctx.input.mouse_drag_delta().x / 4.0;
			}
		}

		self.update_velocity();
		self.update_dragging(ctx, state, idx);
	}
	fn update_dragging(&mut self, ctx: &mut AppContext, state: &mut CarouselState, idx: usize) {
		// Whether the cartridge can be dragged & dropped
		let accessible = self.pos_z < 0.3;

		if self.is_trying_to_drag {
			if !ctx.input.left_is_pressed() || state.is_dragging_any() {
				self.is_trying_to_drag = false;
			}

			if ctx.input.mouse_drag_delta().x.abs() > 40.0 {
				self.start_drag(state, idx);
			}
		}

		if accessible && !state.is_dragging_any() {
			if self.sprite.is_hover(&mut ctx.input) {
				self.anim.update(&ctx.time);

				if ctx.input.left_just_pressed() {
					self.is_trying_to_drag = true;
				}
			}
		}
	}
	fn update_velocity(&mut self) {
		// Tilt the sprite a little on movement
		let angle = (self.pos.x - self.prev_pos.x) / 20.0;
		self.angle_velocity += angle;

		if self.angle_velocity.abs() <= 1e-5 {
			self.angle_velocity = 0.0;
			return;
		}

		self.angle_velocity += (0.0 - self.sprite.angle) / 20.0;
		self.angle_velocity *= 0.9;
		self.sprite.angle += self.angle_velocity;
	}

	fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		let p = (-0.2_f32).lerp(2.0_f32, self.pos_z);
		let light = (1.0 - p).clamp(0.0, 1.0).snap_floor(0.1);

		if light <= 0.0 {
			return;
		}

		// Draw sprite
		let scale = 1.0 / (self.pos_z + 1.0);
		self.sprite.size = ctx.assets.cartridge.size * scale;
		self.sprite.pos = (self.pos - self.sprite.size / 2.0).floor();
		self.sprite.foreground = Color::gray(light);
		self.sprite.frame.x = self.anim.frame;
		self.sprite.draw(&mut ctx.painter, canvas);

		// Draw text
		let mut text = Text::new(&ctx.assets.ibm_font).with_fg(Color::gray(light));

		let bytes = self.info.title.as_bytes();
		let width = bytes.len() as f32 * text.char_size().x;

		text.pos.x = self.sprite.pos.x + self.sprite.size.x / 2.0 - width / 2.0;
		text.pos.y = self.sprite.pos.y + self.sprite.size.y / 3.0;
		text.pos = text.pos.floor();

		text.draw_chars(&mut ctx.painter, canvas, bytes);
	}
}

/// Cartridges carousel
pub struct Carousel {
	pub state: CarouselState,

	cards: Vec<Card>,
	sorted_cards: Vec<usize>,

	angle: f32,
	velocity: f32,
}
impl Carousel {
	const HEIGHT: f32 = 300.0;
	const POS: Point = Point::new(100.0, CANVAS_WIDTH / 2.0);
	/// Angle between each cartridge sprite (in radians)
	const ANGLE_BETWEEN: f32 = PI / (GAMES.len() as f32 / 2.0);

	pub fn new(ctx: &mut AppContext) -> Self {
		let mut cards = Vec::with_capacity(GAMES.len());

		for info in GAMES {
			cards.push(Card::new(ctx, info))
		}

		Self {
			state: CarouselState::default(),

			sorted_cards: (0..cards.len()).collect(),
			cards,

			angle: 0.0,
			velocity: 0.0,
		}
	}

	fn sort_cards(&mut self) {
		// Sort cartridges by Z position
		self.sorted_cards.sort_by(|ia, ib| {
			let a = self.cards[*ia].pos_z;
			let b = self.cards[*ib].pos_z;

			b.partial_cmp(&a).unwrap_or(Ordering::Equal)
		});
	}

	pub fn update(&mut self, ctx: &mut AppContext) {
		const COUNT: f32 = GAMES.len() as f32;

		self.update_velocity(ctx);

		// Place cartridge sprites in circle
		for (idx, card) in self.cards.iter_mut().enumerate() {
			let a = self.angle + idx as f32 / COUNT * TAU + Self::ANGLE_BETWEEN / 2.0;

			let x = Self::POS.x;
			let y = Self::POS.y + a.sin() * Self::HEIGHT;
			let z = (a.cos() + 1.0) / 2.0;

			card.update(ctx, &mut self.state, idx, (x, y).into(), z);
		}

		// Sort sprites every 2nd frame to reduce overhead
		// This solution is kinda bad, but it seems work
		if ctx.time.elapsed % 2 == 0 {
			self.sort_cards();
		}

		self.state.just_dragged = false;
	}
	fn update_velocity(&mut self, ctx: &mut AppContext) {
		let is_rotating = ctx.input.left_is_pressed() && ctx.input.mouse_pos.x < CANVAS_WIDTH / 2.0;
		if !self.state.is_dragging_any() && is_rotating {
			self.velocity = -ctx.input.mouse_movement.y / CANVAS_HEIGHT * PI;
		} else {
			let snap_to_angle = self.angle.snap_round(Self::ANGLE_BETWEEN);
			self.velocity += (snap_to_angle - self.angle) / 100.0;
		}

		self.velocity *= 0.95;
		if self.velocity.abs() <= 1e-5 {
			self.velocity = 0.0;
		}

		self.angle += self.velocity;
		self.angle %= TAU;
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		for i in self.sorted_cards.iter() {
			self.cards[*i].draw(ctx, canvas);
		}
	}
}
