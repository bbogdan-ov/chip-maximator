//! SCOLOC card game
//! Scoundrel with slightly modified rules
//!
//! Inspired by simple but super fun card games:
//! - http://www.stfj.net/art/2011/Scoundrel.pdf
//! - https://100r.co/site/donsol.html

use std::time::Duration;

use quad_rand::ChooseRandom;

use crate::{
	app::AppContext,
	math::{Color, Lerp, Point, Rect, ToStrBytes},
	painter::{CanvasId, Icon, IconKind, Sprite, Text},
	util::{Easing, Timer, Tweenable},
};

use super::{Screen, TitlesContext, TitlesDisplay};

const DECK_CARDS: usize = 52 - 8;
const ROOM_CARDS: usize = 4;
const MAX_HEALTH: u8 = 20;

const RULES: &str = include_str!("../../../assets/text/scoloc-rules.txt");
const RULES_LINES: usize = {
	// Count number of newline chars
	let bytes = RULES.as_bytes();
	let mut num = 0;
	let mut i = 0;
	while i < bytes.len() {
		if bytes[i] == b'\n' {
			num += 1;
		}
		i += 1;
	}
	num
};

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

/// Alert kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlertKind {
	GameOver,
	Win,
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
	target_pos: Point,

	delay: Timer,
	tween_progress: Tweenable,
	animate_appear: bool,
}
impl CardSprite {
	fn new(ctx: &AppContext, idx: usize) -> Self {
		const GAP: f32 = 10.0;

		let s = Sprite::from(&ctx.assets.card);

		// Stack cards in a 2x2 grid
		let x = 16.0 + (idx % 2) as f32 * (s.size.x + GAP);
		let y = 20.0 + if idx < 2 { 0.0 } else { s.size.y + GAP };

		Self {
			inner: s,
			target_pos: (x, y).into(),

			delay: Timer::default(),
			tween_progress: Tweenable::default(),
			animate_appear: true,
		}
	}

	fn update(&mut self, ctx: &mut AppContext, idx: usize) -> bool {
		self.tween_progress.update(&ctx.time);

		if self.animate_appear && self.delay.finished() {
			// Start delay before playing appear animation
			let dur = Duration::from_millis(300 + idx as u64 * 140);
			self.delay.start_duration(dur);
		}

		self.delay.update(&ctx.time);

		if self.animate_appear {
			self.tween_progress.value = 0.0;

			if self.delay.finished() {
				// Play apper animation when delay is finished
				let dur = Duration::from_millis(300);
				self.tween_progress.play(1.0, dur, Easing::InOutSine);

				self.animate_appear = false;
			}
		}

		let inner = &mut self.inner;

		// Lerp between two positions
		let from = inner.size * -1.0;
		inner.pos = from.lerp(self.target_pos, *self.tween_progress);

		if self.tween_progress.playing() {
			return false;
		}

		if self.inner.is_hover(&mut ctx.input) {
			// Animate floating
			let t = ctx.time.elapsed as f32;
			let sine_x = (t / 8.0).cos() * 2.0;
			let sine_y = (t / 4.0).sin() * 2.0;

			self.inner.pos.x += sine_x;
			self.inner.pos.y += sine_y;

			true
		} else {
			false
		}
	}
	fn update_card(&mut self, card: &Card) {
		self.inner.frame = card.sprite_frame();
		self.animate_appear = true;
	}

	fn draw(&self, ctx: &mut AppContext, canvas: CanvasId) {
		self.inner.draw(&mut ctx.painter, canvas);
	}
}

/// Scoloc game state
struct GameState {
	deck: Vec<Card>,
	room: [Option<Card>; ROOM_CARDS],
	/// Number of cards in the current room
	room_cards: usize,

	health: u8,
	weapon: u8,
	/// Whether the player ran from the previous room
	prev_ran: bool,
	/// Whether the player used potion of the previous step
	used_potion: bool,
	killed_cards: Vec<CardGrade>,

	distorting: bool,
	alert_kind: Option<AlertKind>,
}
impl Default for GameState {
	fn default() -> Self {
		let mut deck: Vec<Card> = DEFAULT_DECK.into();
		deck.shuffle();

		Self {
			deck,
			room: [None; ROOM_CARDS],
			room_cards: 0,

			health: MAX_HEALTH,
			weapon: 0,
			prev_ran: false,
			used_potion: false,
			killed_cards: Vec::with_capacity(14),

			distorting: false,
			alert_kind: None,
		}
	}
}

/// Scoloc card game
pub struct Scoloc {
	state: GameState,

	hovered_card_idx: Option<usize>,
	picked_card_idx: Option<usize>,

	rules_opened: bool,
	alert_tween_y: Tweenable,

	card_sprites: [CardSprite; ROOM_CARDS],
	/// The last texture of the distort canvas
	/// TODO: should be just a texture to which we will copy `distort_canvas` texture data
	distort_canvas_last: CanvasId,
	distort_canvas: CanvasId,
}
impl Scoloc {
	pub fn new(ctx: &mut AppContext) -> Self {
		const SIZE: Point = Point::new(TitlesDisplay::SIZE, TitlesDisplay::SIZE);

		// Populate card sprites
		let mut i = 0;
		let card_sprites = [(); ROOM_CARDS].map(|_| {
			let sprite = CardSprite::new(ctx, i);
			i += 1;
			sprite
		});

		let pctx = &mut ctx.painter.context;
		let mut game = Self {
			state: GameState::default(),

			hovered_card_idx: None,
			picked_card_idx: None,

			rules_opened: false,
			alert_tween_y: Tweenable::new(-TitlesDisplay::SIZE),

			card_sprites,
			distort_canvas_last: pctx.new_canvas_no_clear(SIZE, Default::default()),
			distort_canvas: pctx.new_canvas_no_clear(SIZE, Default::default()),
		};

		game.next_room();
		game
	}

	fn next_room(&mut self) {
		if self.state.deck.is_empty() {
			self.set_alert(AlertKind::Win);
			return;
		}

		// Take last 4 cards from the deck
		let end = self.state.deck.len();
		let start = end.saturating_sub(4);
		let cards = self.state.deck.drain(start..end);

		self.state.room_cards = 0;
		self.state.prev_ran = false;

		// Update card sprites and put taken cards into a new room
		for (i, card) in cards.enumerate() {
			self.card_sprites[i].update_card(&card);
			self.state.room[i] = Some(card);
			self.state.room_cards += 1;
		}
	}
	fn run(&mut self) {
		if self.state.prev_ran {
			return;
		}

		// Put remaining room cards into the bottom of the deck
		for card in self.state.room.iter_mut() {
			if let Some(card) = card.take() {
				self.state.deck.insert(0, card);
			}
		}

		self.next_room();
		self.state.prev_ran = true;
	}
	fn pick_card(&mut self, idx: usize) {
		let Some(card) = self.state.room[idx] else {
			return;
		};

		let value = card.grade.value();
		match card.kind {
			CardKind::Diamonds => self.equip(value),
			CardKind::Hearts => self.heal(value),
			CardKind::Club => self.damage(card.grade),
			CardKind::Spade => self.damage(card.grade),
		}

		self.state.used_potion = card.kind == CardKind::Hearts;
		self.state.room_cards -= 1;
		self.state.room[idx] = None;

		self.picked_card_idx = Some(idx);
		self.state.distorting = true;
	}

	fn equip(&mut self, weapon: u8) {
		self.state.weapon = weapon;
		self.state.killed_cards.clear();
	}
	fn heal(&mut self, value: u8) {
		if self.state.used_potion {
			return;
		}

		self.state.health = (self.state.health + value).min(MAX_HEALTH);
	}
	fn damage(&mut self, grade: CardGrade) {
		let damage: u8;
		let value = grade.value();

		if let Some(monster) = self.state.killed_cards.last() {
			if value < monster.value() {
				// Take no damage
				damage = 0;
				self.state.killed_cards.push(grade);
			} else {
				// Break the weapon and take full damage from the monster
				damage = value;
				self.state.weapon = 0;
				self.state.killed_cards.clear();
			}
		} else {
			// No moster were killed with this weapon before, suppress the damage
			damage = value.saturating_sub(self.state.weapon);
			if damage < value {
				self.state.killed_cards.push(grade);
			}
		}

		self.state.health = self.state.health.saturating_sub(damage);
		if self.state.health == 0 {
			self.set_alert(AlertKind::GameOver);
		}
	}

	fn set_alert(&mut self, kind: AlertKind) {
		self.alert_tween_y.value = -TitlesDisplay::SIZE;
		self.alert_tween_y
			.play(0.0, Duration::from_millis(1000), Easing::Linear);

		self.state.alert_kind = Some(kind);
	}

	pub fn update(&mut self, ctx: &mut AppContext) {
		self.alert_tween_y.update(&ctx.time);

		self.hovered_card_idx = None;

		if self.paused() {
			return;
		}

		// Check for an empty room inside update
		if self.state.room_cards == 0 {
			self.next_room();
		}

		for idx in 0..self.card_sprites.len() {
			if self.state.room[idx].is_none() {
				continue;
			}

			let sprite = &mut self.card_sprites[idx];

			let hovered = sprite.update(ctx, idx);
			if hovered {
				self.hovered_card_idx = Some(idx);

				if ctx.input.left_just_pressed() {
					self.pick_card(idx);
				}
			}
		}
	}
	fn reset(&mut self) {
		self.state = GameState::default();
		self.next_room();
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext) {
		if !self.state.distorting {
			// Clear canvas with an image
			Sprite::from(&ctx.assets.titles_bg).draw(&mut ctx.painter, self.distort_canvas);
			ctx.painter.clear(Some(self.distort_canvas_last));
			return;
		};

		for _ in 0..4 {
			self.distort(ctx);
		}

		if let Some(idx) = self.picked_card_idx.take() {
			self.card_sprites[idx].draw(ctx, self.distort_canvas);
		}

		// Save current `distort_canvas` texture
		Sprite::from(ctx.painter.canvas(self.distort_canvas))
			.draw(&mut ctx.painter, self.distort_canvas_last);
	}
	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId, titles_ctx: &mut TitlesContext) {
		Sprite::from(ctx.painter.canvas(self.distort_canvas)).draw(&mut ctx.painter, canvas);

		self.draw_room(ctx, canvas);

		self.draw_stat(ctx, canvas, 26.0, IconKind::Heart, self.state.health);
		self.draw_stat(ctx, canvas, 56.0, IconKind::Sword, self.state.weapon);
		self.draw_killed_monsters(ctx, canvas);
		self.draw_description(ctx, canvas);

		self.draw_alert(ctx, canvas);
		self.draw_rules(ctx, canvas);

		self.draw_buttons(ctx, canvas, titles_ctx);
	}

	fn distort(&self, ctx: &mut AppContext) {
		use quad_rand::rand;

		const DS: f32 = TitlesDisplay::SIZE;
		const STEP: f32 = 2.0;

		let slices = quad_rand::gen_range(2, 8);

		// Pick a random frame
		let (frame_x, frame_y) = {
			let idx = quad_rand::gen_range(0, slices * slices);
			(idx % slices, idx / slices)
		};

		let offset_x = if rand() % 2 == 0 { STEP } else { -STEP };
		let offset_y = if rand() % 2 == 0 { STEP } else { -STEP };

		// Draw last canvas texture, crop an quarter and offset it by random about of pixels
		let cvs = ctx.painter.canvas(self.distort_canvas_last);
		let mut sprite = Sprite::from(cvs)
			.with_pos((
				(DS / slices as f32) * frame_x as f32 + offset_x,
				(DS / slices as f32) * frame_y as f32 + offset_y,
			))
			.with_frames_count((slices, slices))
			.with_frame((frame_x, frame_y))
			.with_scale(1.0 / slices as f32);

		if self.is_game_over() {
			sprite.foreground = Color::new(1.0, 0.99, 0.99);
		}

		sprite.draw(&mut ctx.painter, self.distort_canvas);
	}

	fn draw_stat(&self, ctx: &mut AppContext, canvas: CanvasId, y: f32, icon: IconKind, num: u8) {
		const DS: f32 = TitlesDisplay::SIZE;
		const X: f32 = DS - 64.0;

		Icon::new(&ctx.assets, icon)
			.with_pos((X, y + 16.0))
			.draw(ctx, canvas);

		Text::new(&ctx.assets.serif_font)
			.with_fg(Color::BLACK)
			.with_bg(Color::TRANSPARENT)
			.with_font_size(1.0)
			.with_pos((X + 16.0, y))
			.draw_chars(&mut ctx.painter, canvas, &num.to_str_bytes());
	}
	fn draw_killed_monsters(&self, ctx: &mut AppContext, canvas: CanvasId) {
		const DS: f32 = TitlesDisplay::SIZE;
		const X: f32 = DS - 70.0;
		const Y: f32 = 94.0;

		for (i, grade) in self.state.killed_cards.iter().enumerate() {
			let frame = grade.value() as i32 - 2;
			let mut sprite = Sprite::from(&ctx.assets.small_card).with_frame((frame, 0));
			sprite.pos.set(X, Y + i as f32 * 10.0);
			sprite.draw(&mut ctx.painter, canvas);
		}
	}
	fn draw_buttons(
		&mut self,
		ctx: &mut AppContext,
		canvas: CanvasId,
		titles_ctx: &mut TitlesContext,
	) {
		const DS: f32 = TitlesDisplay::SIZE;

		const BTN_W: f32 = 16.0 * 3.0;
		const BTN_H: f32 = 16.0;

		const RUN_BTN: Rect = Rect::new_xywh(0.0, 0.0, 16.0 * 5.0, BTN_H);
		const CLOSE_BTN: Rect = Rect::new_xywh(DS - BTN_W, 0.0, BTN_W, BTN_H);
		const TUTORIAL_BTN: Rect = Rect::new_xywh(DS - BTN_W * 2.0, 0.0, BTN_W, BTN_H);

		if CLOSE_BTN.is_hover(&mut ctx.input) && ctx.input.left_just_pressed() {
			if self.rules_opened {
				self.rules_opened = false;
			} else {
				titles_ctx.goto_screen(Screen::default());
				self.reset();
			}
		}
		if TUTORIAL_BTN.is_hover(&mut ctx.input) && ctx.input.left_just_pressed() {
			self.rules_opened ^= true;
		}

		if !self.state.prev_ran && !self.paused() {
			if RUN_BTN.is_hover(&mut ctx.input) && ctx.input.left_just_pressed() {
				self.run();
			}

			// Run button
			Text::new(&ctx.assets.ibm_font)
				.with_pos(RUN_BTN.pos)
				.with_font_size(2.0)
				.draw_chars(&mut ctx.painter, canvas, b"[run]");
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
		for (i, sprite) in self.card_sprites.iter_mut().enumerate() {
			let Some(card) = self.state.room[i] else {
				continue;
			};

			// Dim hearts cards if potion was used on the prev step
			if self.state.used_potion && card.kind == CardKind::Hearts {
				sprite.inner.foreground = Color::gray(0.5);
			} else {
				sprite.inner.foreground = Color::WHITE;
			}

			sprite.draw(ctx, canvas);
		}
	}
	fn draw_description(&self, ctx: &mut AppContext, canvas: CanvasId) {
		const DS: f32 = TitlesDisplay::SIZE;

		// Draw currently hovered card name and value
		if let Some(idx) = self.hovered_card_idx {
			let Some(card) = &self.state.room[idx] else {
				return;
			};

			Text::new(&ctx.assets.serif_font)
				.with_fg(Color::BLACK)
				.with_bg(Color::TRANSPARENT)
				.with_pos((16.0, DS - 40.0))
				.draw_chars(&mut ctx.painter, canvas, card.kind.name().as_bytes())
				.draw_chars(&mut ctx.painter, canvas, b" - ")
				.draw_chars(&mut ctx.painter, canvas, &card.grade.value().to_str_bytes());
		}
	}

	fn draw_alert(&self, ctx: &mut AppContext, canvas: CanvasId) {
		let Some(kind) = self.state.alert_kind else {
			return;
		};

		const DS: f32 = TitlesDisplay::SIZE;
		const Y: f32 = DS - 78.0;

		let (title, subtitle, bg) = match kind {
			AlertKind::GameOver => ("GAME OVER", " oh no ", Color::new(1.0, 0.0, 0.0)),
			AlertKind::Win => ("GREAT", " yeah ", Color::new(0.0, 1.0, 0.0)),
		};

		Text::new(&ctx.assets.serif_font)
			.with_pos((16.0, Y + *self.alert_tween_y))
			.with_fg(Color::BLACK)
			.with_bg(bg)
			.draw_chars(&mut ctx.painter, canvas, title.as_bytes());

		Text::new(&ctx.assets.ibm_font)
			.with_font_size(2.0)
			.with_pos((16.0, Y + 46.0 + *self.alert_tween_y))
			.draw_chars(&mut ctx.painter, canvas, subtitle.as_bytes());
	}
	fn draw_rules(&self, ctx: &mut AppContext, canvas: CanvasId) {
		const DS: f32 = TitlesDisplay::SIZE;
		const PADDING: f32 = 24.0;
		const FONT_SIZE: f32 = 0.65;

		if !self.rules_opened {
			return;
		}

		let serif = &ctx.assets.serif_font;

		let scroll_h = RULES_LINES as f32 * (serif.size.y * FONT_SIZE);
		let scrollf = (ctx.input.mouse_pos.y / DS).clamp(0.0, 1.0);
		let scroll = ((scroll_h - DS + PADDING).max(0.0) * scrollf).floor();

		Sprite::new(ctx.painter.white_texture, (DS, DS))
			.with_fg(Color::hex(0xba1062))
			.draw(&mut ctx.painter, canvas);

		Text::new(serif)
			.with_pos((6.0, PADDING - scroll))
			.with_font_size(FONT_SIZE)
			.with_bg(Color::TRANSPARENT)
			.draw_str(&mut ctx.painter, canvas, RULES);
	}

	fn paused(&self) -> bool {
		self.state.alert_kind.is_some() || self.rules_opened
	}
	fn is_game_over(&self) -> bool {
		self.state
			.alert_kind
			.is_some_and(|k| k == AlertKind::GameOver)
	}
}
