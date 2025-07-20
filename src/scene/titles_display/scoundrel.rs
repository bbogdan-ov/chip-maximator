//! SCOUNDREL card game
//! This is a simple but a super fun card game
//!
//! You can read the rules here: http://www.stfj.net/art/2011/Scoundrel.pdf

use crate::{
	app::AppContext,
	math::{Point, ToStrBytes},
	painter::{CanvasId, Sprite, Text},
};

use super::TitlesDisplay;

const DECK_CARDS: usize = 52 - 8;
const ROOM_CARDS: usize = 4;

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

	fn name(&self) -> &'static str {
		self.kind.name()
	}
	fn value(&self) -> u8 {
		self.grade.value()
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

	fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		self.inner.draw(&mut ctx.painter, canvas);
	}
}

/// Scoundrel card game
pub struct Scoundrel {
	deck: [Option<Card>; DECK_CARDS],
	room: Option<[Card; ROOM_CARDS]>,

	card_sprites: [CardSprite; ROOM_CARDS],
}
impl Scoundrel {
	pub fn new(ctx: &AppContext) -> Self {
		// Populate card sprites
		let card_sprites = [(); ROOM_CARDS].map(|_| CardSprite::new(&ctx));

		let mut game = Self {
			deck: DEFAULT_DECK.map(|c| Some(c)),
			room: None,

			card_sprites,
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

		self.room = Some(cards);
	}

	pub fn update(&mut self, ctx: &mut AppContext) {}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		self.draw_room(ctx, canvas);
	}
	fn draw_room(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		const GAP: f32 = 16.0;
		const DS: f32 = TitlesDisplay::SIZE;

		let Some(ref room) = self.room else { return };

		let mut hovered_card: Option<&Card> = None;

		for (i, sprite) in self.card_sprites.iter_mut().enumerate() {
			let card = &room[i];

			// Stack cards in a 2x2 grid
			let inner = &mut sprite.inner;
			inner.pos.set(GAP, GAP);
			inner.pos.x += (i % 2) as f32 * (inner.size.x + GAP);
			inner.pos.y += if i < 2 { 0.0 } else { inner.size.y + GAP };

			if sprite.inner.is_hover(&mut ctx.input) {
				hovered_card = Some(card);
				sprite.inner.pos.y += -4.0;
			}

			sprite.draw(ctx, canvas);
		}

		// Draw currently hovered card name and value
		if let Some(card) = hovered_card {
			let name = card.name();
			let value = card.value();

			Text::new(&ctx.assets.ibm_font)
				.with_font_size(2.0)
				.with_pos((GAP, DS - 24.0))
				.draw_chars(&mut ctx.painter, canvas, name.as_bytes())
				.draw_chars(&mut ctx.painter, canvas, b" - ")
				.draw_chars(&mut ctx.painter, canvas, &value.to_str_bytes());
		}
	}
}
