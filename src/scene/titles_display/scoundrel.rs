//! SCOUNDREL card game
//! This is a simple but a super fun card game
//!
//! You can read the rules here: http://www.stfj.net/art/2011/Scoundrel.pdf

use crate::{
	app::AppContext,
	math::{Color, Point, Rect, ToStrBytes},
	painter::{CanvasId, Icon, IconKind, Sprite, Text},
};

use super::TitlesDisplay;

const DECK_CARDS: usize = 52 - 8;
const ROOM_CARDS: usize = 4;
const MAX_HEALTH: u8 = 20;

#[rustfmt::skip]
const DEFAULT_DECK: [Card; DECK_CARDS] = [
	Card::new(CardKind::Spade, CardGrade::Two),
	Card::new(CardKind::Spade, CardGrade::Three),
	Card::new(CardKind::Spade, CardGrade::Four),
	Card::new(CardKind::Spade, CardGrade::Five),
	Card::new(CardKind::Spade, CardGrade::Six),
	Card::new(CardKind::Spade, CardGrade::Seven),
	Card::new(CardKind::Spade, CardGrade::Eight),
	Card::new(CardKind::Spade, CardGrade::Nine),
	Card::new(CardKind::Spade, CardGrade::Ten),
	Card::new(CardKind::Spade, CardGrade::Jack),
	Card::new(CardKind::Spade, CardGrade::Queen),
	Card::new(CardKind::Spade, CardGrade::King),
	Card::new(CardKind::Spade, CardGrade::Ace),

	Card::new(CardKind::Club, CardGrade::Two),
	Card::new(CardKind::Club, CardGrade::Three),
	Card::new(CardKind::Club, CardGrade::Four),
	Card::new(CardKind::Club, CardGrade::Five),
	Card::new(CardKind::Club, CardGrade::Six),
	Card::new(CardKind::Club, CardGrade::Seven),
	Card::new(CardKind::Club, CardGrade::Eight),
	Card::new(CardKind::Club, CardGrade::Nine),
	Card::new(CardKind::Club, CardGrade::Ten),
	Card::new(CardKind::Club, CardGrade::Jack),
	Card::new(CardKind::Club, CardGrade::Queen),
	Card::new(CardKind::Club, CardGrade::King),
	Card::new(CardKind::Club, CardGrade::Ace),

	Card::new(CardKind::Diamonds, CardGrade::Two),
	Card::new(CardKind::Diamonds, CardGrade::Three),
	Card::new(CardKind::Diamonds, CardGrade::Four),
	Card::new(CardKind::Diamonds, CardGrade::Five),
	Card::new(CardKind::Diamonds, CardGrade::Six),
	Card::new(CardKind::Diamonds, CardGrade::Seven),
	Card::new(CardKind::Diamonds, CardGrade::Eight),
	Card::new(CardKind::Diamonds, CardGrade::Nine),
	Card::new(CardKind::Diamonds, CardGrade::Ten),

	Card::new(CardKind::Hearts, CardGrade::Two),
	Card::new(CardKind::Hearts, CardGrade::Three),
	Card::new(CardKind::Hearts, CardGrade::Four),
	Card::new(CardKind::Hearts, CardGrade::Five),
	Card::new(CardKind::Hearts, CardGrade::Six),
	Card::new(CardKind::Hearts, CardGrade::Seven),
	Card::new(CardKind::Hearts, CardGrade::Eight),
	Card::new(CardKind::Hearts, CardGrade::Nine),
	Card::new(CardKind::Hearts, CardGrade::Ten),
];

/// Card grade
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CardGrade {
	Two,
	Three,
	Four,
	Five,
	Six,
	Seven,
	Eight,
	Nine,
	Ten,
	Jack,
	Queen,
	King,
	Ace,
}
impl CardGrade {
	fn value(&self) -> u8 {
		match self {
			Self::Two => 2,
			Self::Three => 3,
			Self::Four => 4,
			Self::Five => 5,
			Self::Six => 6,
			Self::Seven => 7,
			Self::Eight => 8,
			Self::Nine => 9,
			Self::Ten => 10,
			Self::Jack => 11,
			Self::Queen => 12,
			Self::King => 13,
			Self::Ace => 14,
		}
	}
}

/// Card kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CardKind {
	/// Weapon
	Diamonds,
	/// Heal potion
	Hearts,
	/// Monster
	Club,
	/// Monster
	Spade,
}
impl CardKind {
	fn name(&self) -> &'static str {
		match self {
			Self::Diamonds => "weapon",
			Self::Hearts => "potion",
			Self::Club => "monster",
			Self::Spade => "monster",
		}
	}
}

/// Card
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Card {
	kind: CardKind,
	grade: CardGrade,
}
impl Card {
	const fn new(kind: CardKind, grade: CardGrade) -> Self {
		Self { kind, grade }
	}

	fn sprite_frame(&self) -> Point<i32> {
		let x = self.grade.value() as i32 - 2;
		let y = match self.kind {
			CardKind::Diamonds => 3,
			CardKind::Hearts => 1,
			CardKind::Club => 2,
			CardKind::Spade => 0,
		};

		(x, y).into()
	}
}

/// Card sprite
struct CardSprite {
	inner: Sprite,
}
impl CardSprite {
	fn new(ctx: &AppContext) -> Self {
		Self {
			inner: Sprite::from(&ctx.assets.card),
		}
	}

	fn update_card(&mut self, card: &Card) {
		self.inner.frame = card.sprite_frame();
	}

	fn draw(&self, ctx: &mut AppContext, canvas: CanvasId) {
		self.inner.draw(&mut ctx.painter, canvas);
	}
}

/// Scoundrel card game
pub struct Scoundrel {
	deck: [Option<Card>; DECK_CARDS],
	room: [Option<Card>; ROOM_CARDS],
	room_cards: usize,

	health: u8,
	weapon: u8,

	hovered_card_idx: Option<usize>,
	picked_sprite_idx: Option<usize>,

	card_sprites: [CardSprite; ROOM_CARDS],
	distort_canvas: CanvasId,
}
impl Scoundrel {
	pub fn new(ctx: &mut AppContext) -> Self {
		// Populate card sprites
		let card_sprites = [(); ROOM_CARDS].map(|_| CardSprite::new(&ctx));

		let mut game = Self {
			deck: DEFAULT_DECK.map(Some),
			room: [None; ROOM_CARDS],
			room_cards: 0,

			health: MAX_HEALTH,
			weapon: 0,

			hovered_card_idx: None,
			picked_sprite_idx: None,

			card_sprites,
			distort_canvas: ctx.painter.context.new_canvas_no_clear(
				(TitlesDisplay::SIZE, TitlesDisplay::SIZE),
				Default::default(),
			),
		};

		game.set_room([
			DEFAULT_DECK[0],
			DEFAULT_DECK[20],
			DEFAULT_DECK[43],
			DEFAULT_DECK[31],
		]);

		game
	}

	fn set_room(&mut self, cards: [Card; ROOM_CARDS]) {
		for (i, card) in cards.iter().enumerate() {
			self.card_sprites[i].update_card(card);
		}

		self.room = cards.map(Some);
		self.room_cards = ROOM_CARDS;
	}
	fn pick_card(&mut self, idx: usize) {
		let card = self.room[idx]
			.as_ref()
			.unwrap_or_else(|| panic!("no card at {idx}"));

		let value = card.grade.value();

		match card.kind {
			CardKind::Diamonds => self.equip(value),
			CardKind::Hearts => self.heal(value),
			CardKind::Club => self.damage(value),
			CardKind::Spade => self.damage(value),
		}

		self.room[idx] = None;
		self.room_cards -= 1;
	}

	fn equip(&mut self, weapon: u8) {
		self.weapon = weapon;
	}
	fn heal(&mut self, value: u8) {
		self.health = (self.health + value).min(MAX_HEALTH);
	}
	fn damage(&mut self, value: u8) {
		self.health = self.health.saturating_sub(value);
	}

	pub fn update(&mut self, ctx: &mut AppContext) {
		const GAP: f32 = 10.0;

		self.hovered_card_idx = None;

		for idx in 0..self.card_sprites.len() {
			if self.room[idx].is_none() {
				continue;
			}

			let sprite = &mut self.card_sprites[idx];

			// Stack cards in a 2x2 grid
			let inner = &mut sprite.inner;
			inner.pos.set(16.0, 20.0);
			inner.pos.x += (idx % 2) as f32 * (inner.size.x + GAP);
			inner.pos.y += if idx < 2 { 0.0 } else { inner.size.y + GAP };

			if inner.is_hover(&mut ctx.input) {
				// Animate floating
				let t = ctx.time.elapsed as f32;
				let sine_x = (t / 8.0).cos() * 2.0;
				let sine_y = (t / 4.0).sin() * 2.0;

				sprite.inner.pos.x += sine_x;
				sprite.inner.pos.y += sine_y;

				self.hovered_card_idx = Some(idx);

				if ctx.input.left_just_pressed() {
					self.pick_card(idx);
					self.picked_sprite_idx = Some(idx);
				}
			}
		}
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		use quad_rand::rand;

		const DS: f32 = TitlesDisplay::SIZE;
		const STEP: f32 = 2.0;

		if self.room_cards >= ROOM_CARDS {
			// Clear canvas with an image
			Sprite::from(&ctx.assets.titles_bg).draw(&mut ctx.painter, self.distort_canvas);
			return;
		};

		for _ in 0..4 {
			let slices = quad_rand::gen_range(2, 8);

			// Pick a random frame
			let (frame_x, frame_y) = {
				let idx = quad_rand::gen_range(0, slices * slices);
				(idx % slices, idx / slices)
			};

			let offset_x = if rand() % 2 == 0 { STEP } else { -STEP };
			let offset_y = if rand() % 2 == 0 { STEP } else { -STEP };

			// Draw canvas on itself, crop an quarter and offset it by random about of pixels
			Sprite::from(ctx.painter.canvas(self.distort_canvas))
				.with_pos((
					(DS / slices as f32) * frame_x as f32 + offset_x,
					(DS / slices as f32) * frame_y as f32 + offset_y,
				))
				.with_frames_count((slices, slices))
				.with_frame((frame_x, frame_y))
				.with_scale(1.0 / slices as f32)
				.draw(&mut ctx.painter, self.distort_canvas);
		}

		if let Some(idx) = self.picked_sprite_idx.take() {
			// Draw only the currently hovering card sprite
			self.card_sprites[idx].draw(ctx, self.distort_canvas);
		}
	}
	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		Sprite::from(ctx.painter.canvas(self.distort_canvas)).draw(&mut ctx.painter, canvas);

		self.draw_room(ctx, canvas);

		self.draw_stat(ctx, canvas, 40.0, IconKind::Heart, self.health);
		self.draw_stat(ctx, canvas, 64.0, IconKind::Sword, self.weapon);
		self.draw_description(ctx, canvas);

		self.draw_buttons(ctx, canvas);
	}

	fn draw_stat(&self, ctx: &mut AppContext, canvas: CanvasId, y: f32, icon: IconKind, num: u8) {
		const DS: f32 = TitlesDisplay::SIZE;
		const X: f32 = DS - 52.0;

		Icon::new(&ctx.assets, icon)
			.with_pos((X, y + 8.0))
			.draw(ctx, canvas);

		Text::new(&ctx.assets.serif_font)
			.with_fg(Color::BLACK)
			.with_bg(Color::TRANSPARENT)
			.with_font_size(0.6)
			.with_pos((X + 12.0, y))
			.draw_chars(&mut ctx.painter, canvas, &num.to_str_bytes());
	}
	fn draw_buttons(&self, ctx: &mut AppContext, canvas: CanvasId) {
		const DS: f32 = TitlesDisplay::SIZE;

		const BTN_W: f32 = 16.0 * 3.0;
		const BTN_H: f32 = 16.0;
		const CLOSE_BTN: Rect = Rect::new_xywh(DS - BTN_W, 0.0, BTN_W, BTN_H);
		const TUTORIAL_BTN: Rect = Rect::new_xywh(DS - BTN_W * 2.0, 0.0, BTN_W, BTN_H);

		if CLOSE_BTN.is_hover(&mut ctx.input) && ctx.input.left_just_pressed() {
			println!("close");
		}
		if TUTORIAL_BTN.is_hover(&mut ctx.input) && ctx.input.left_just_pressed() {
			println!("tutor");
		}

		// Close button
		Text::new(&ctx.assets.ibm_font)
			.with_pos(CLOSE_BTN.pos)
			.with_font_size(2.0)
			.draw_chars(&mut ctx.painter, canvas, b"[x]");

		// Tutorial button
		Text::new(&ctx.assets.ibm_font)
			.with_pos(TUTORIAL_BTN.pos)
			.with_font_size(2.0)
			.draw_chars(&mut ctx.painter, canvas, b"[?]");
	}
	fn draw_room(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		for (i, sprite) in self.card_sprites.iter().enumerate() {
			if self.room[i].is_none() {
				continue;
			}

			sprite.draw(ctx, canvas);
		}
	}
	fn draw_description(&self, ctx: &mut AppContext, canvas: CanvasId) {
		const DS: f32 = TitlesDisplay::SIZE;

		// Draw currently hovered card name and value
		if let Some(idx) = self.hovered_card_idx {
			let Some(card) = &self.room[idx] else { return };

			Text::new(&ctx.assets.serif_font)
				.with_fg(Color::BLACK)
				.with_bg(Color::TRANSPARENT)
				.with_pos((16.0, DS - 40.0))
				.draw_chars(&mut ctx.painter, canvas, card.kind.name().as_bytes())
				.draw_chars(&mut ctx.painter, canvas, b" - ")
				.draw_chars(&mut ctx.painter, canvas, &card.grade.value().to_str_bytes());
		}
	}
}
