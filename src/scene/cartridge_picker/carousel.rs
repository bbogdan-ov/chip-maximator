use std::{
	cmp::Ordering,
	f32::consts::{PI, TAU},
};

use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	games::{GAMES, GameInfo},
	math::{Color, FloatMath, Lerp, Point},
	painter::{CanvasId, Sprite, Text},
	util::Anim,
};

/// Cartridge card sprite
struct Card {
	info: &'static GameInfo,

	pos: Point,
	pos_z: f32,
	selected: bool,

	anim: Anim,
}
impl Card {
	fn new(info: &'static GameInfo) -> Self {
		Self {
			info,

			pos: Point::default(),
			pos_z: 0.0,
			selected: false,

			anim: Anim::new(8, 8..16).with_looped().with_playing(),
		}
	}

	fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		let p = (-0.2_f32).lerp(2.0_f32, self.pos_z);
		let light = (1.0 - p).clamp(0.0, 1.0).snap_floor(0.1);

		if light <= 0.0 {
			return;
		}

		// Draw sprite
		let scale = 1.0 / (self.pos_z + 1.0);

		if self.selected {
			self.anim.update(&ctx.time);
		}

		let mut sprite = Sprite::from(&ctx.assets.cartridge)
			.with_pos(self.pos)
			.with_scale(scale)
			.with_fg(Color::gray(light))
			.with_anim(&self.anim);

		sprite.pos.x -= sprite.size.x / 2.0;
		sprite.pos.y -= sprite.size.y / 2.0;
		sprite.pos = sprite.pos.floor();
		sprite.draw(&mut ctx.painter, canvas);

		// Draw text
		let mut text = Text::new(&ctx.assets.ibm_font).with_fg(Color::gray(light));

		let bytes = self.info.title.as_bytes();
		let width = bytes.len() as f32 * text.char_size().x;

		text.pos.x = sprite.pos.x + sprite.size.x / 2.0 - width / 2.0;
		text.pos.y = sprite.pos.y + sprite.size.y / 3.0;
		text.pos = text.pos.floor();

		text.draw_chars(&mut ctx.painter, canvas, bytes);
	}
}

/// Cartridges carousel
pub struct Carousel {
	cards: Vec<Card>,
	sorted_cards: Vec<usize>,

	angle: f32,
	velocity: f32,
	angle_before_sort: f32,
}
impl Carousel {
	const HEIGHT: f32 = 300.0;
	const POS: Point = Point::new(100.0, CANVAS_WIDTH / 2.0);
	/// Angle between each cartridge sprite (in radians)
	const ANGLE_BETWEEN: f32 = PI / (GAMES.len() as f32 / 2.0);

	pub fn new() -> Self {
		let mut cards = Vec::with_capacity(GAMES.len());

		for info in GAMES {
			cards.push(Card::new(info))
		}

		Self {
			sorted_cards: (0..cards.len()).collect(),
			cards,

			angle: 0.0,
			velocity: 0.0,
			// Set it to max to trigger the first sorting in `update`
			angle_before_sort: f32::MAX,
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
		for (i, card) in self.cards.iter_mut().enumerate() {
			let idx = i as f32;
			let a = self.angle + idx / COUNT * TAU + Self::ANGLE_BETWEEN / 2.0;

			card.pos_z = (a.cos() + 1.0) / 2.0;
			card.pos.x = Self::POS.x;
			card.pos.y = Self::POS.y + a.sin() * Self::HEIGHT;
			card.selected = false;
		}

		// Sort sprites only when angle was changed so that sprites Z pos changed significantly
		if (self.angle - self.angle_before_sort).abs() >= Self::ANGLE_BETWEEN {
			self.sort_cards();
			self.angle_before_sort = self.angle;
		}

		if let Some(i) = self.sorted_cards.last() {
			let card = &mut self.cards[*i];
			card.selected = true;
		}
	}
	fn update_velocity(&mut self, ctx: &mut AppContext) {
		if ctx.input.left_is_pressed() {
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
