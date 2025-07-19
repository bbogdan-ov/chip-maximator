use std::{cmp::Ordering, f32};

use crate::{
	app::{AppContext, CANVAS_HEIGHT, CANVAS_WIDTH},
	games::{GAMES, GameInfo},
	math::{Color, FloatMath, Lerp, Point},
	painter::{CanvasId, Icon, IconKind, Sprite, Text},
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
		text.pos.y = text.pos.y.floor();

		text.draw_chars(&mut ctx.painter, canvas, bytes);

		// Draw pointer icon
		if self.selected {
			Icon::new(&ctx.assets, IconKind::Pointer)
				.with_pos((self.pos.x + 100.0, self.pos.y))
				.draw(ctx, canvas);

			Text::new(&ctx.assets.ibm_font)
				.with_pos((self.pos.x + 150.0, self.pos.y))
				.draw_str(&mut ctx.painter, canvas, self.info.desc);
		}
	}
}

/// Cartridge picker
pub struct CartridgePicker {
	cards: Vec<Card>,
	sorted_cards: Vec<usize>,

	angle: f32,
	velocity: f32,
}
impl Default for CartridgePicker {
	fn default() -> Self {
		let mut cards = Vec::with_capacity(GAMES.len());

		for info in GAMES {
			cards.push(Card::new(info))
		}

		Self {
			sorted_cards: (0..cards.len()).collect(),
			cards,

			angle: 0.0,
			velocity: 0.0,
		}
	}
}
impl CartridgePicker {
	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		const PI: f32 = f32::consts::PI;
		const PI2: f32 = f32::consts::TAU;

		// Draw darken rect
		Sprite::new(ctx.painter.white_texture, (CANVAS_WIDTH, CANVAS_HEIGHT))
			.with_fg((0.0, 0.0, 0.0))
			.with_opacity(0.8)
			.draw(&mut ctx.painter, canvas);

		let count = self.cards.len();
		let n = count as f32;
		let step = PI / (n / 2.0);

		if ctx.input.left_is_pressed() {
			self.velocity = -ctx.input.mouse_movement.y / CANVAS_HEIGHT * PI;
		} else {
			self.velocity += (self.angle.snap_round(step) - self.angle) / 100.0;
		}

		self.velocity *= 0.95;
		if self.velocity.abs() <= 1e-5 {
			self.velocity = 0.0;
		}

		self.angle += self.velocity;
		self.angle %= PI2;

		for (i, card) in self.cards.iter_mut().enumerate() {
			card.pos_z = ((i as f32 / n * PI2 + self.angle + step / 2.0).cos() + 1.0) / 2.0;
			card.pos.x = 140.0;
			card.pos.y =
				(i as f32 / n * PI2 + self.angle + step / 2.0).sin() * 300.0 + CANVAS_WIDTH / 2.0;
			card.selected = false;
		}

		self.sorted_cards.sort_by(|ia, ib| {
			let a = &self.cards[*ia].pos_z;
			let b = &self.cards[*ib].pos_z;

			b.partial_cmp(a).unwrap_or(Ordering::Equal)
		});

		if let Some(i) = self.sorted_cards.last() {
			let card = &mut self.cards[*i];
			card.selected = true;
		}

		for i in self.sorted_cards.iter() {
			self.cards[*i].draw(ctx, canvas);
		}
	}
}
