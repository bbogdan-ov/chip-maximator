use std::time::Duration;

use crate::{
	app::AppContext,
	math::{Color, Lerp, Point, Rect},
	painter::{CanvasId, Sprite, Text},
	scene::titles_display::TitlesDisplay,
	util::{Anim, Easing, Timer, Tweenable},
};

use super::{Screen, TitlesContext};

fn draw_icon_text(ctx: &mut AppContext, canvas: CanvasId, icon_rect: Rect, text: &[u8]) {
	const SCALE: f32 = 1.3;

	let width = text.len() as f32 * (8.0 * SCALE);
	let offset = Point::new(icon_rect.size.x / 2.0 - width / 2.0, icon_rect.size.y + 4.0);

	Text::new(&ctx.assets.ibm_font)
		.with_pos(icon_rect.pos + offset)
		.with_font_size(SCALE)
		.with_bg(Color::new(0.0, 0.0, 1.0))
		.draw_chars(&mut ctx.painter, canvas, text);
}

/// Clock
struct Clock {
	sprite: Sprite,
	anim: Anim,

	physics_timer: Timer,
	velocity: Point,
	tween_progress: Tweenable,
}
impl Clock {
	const POS: Point = {
		const DS: f32 = TitlesDisplay::SIZE;
		Point::new(84.0, DS - 78.0)
	};

	fn new(ctx: &mut AppContext) -> Self {
		Self {
			sprite: Sprite::from(&ctx.assets.clock).with_pos(Self::POS),
			anim: Anim::new(8, 0..ctx.assets.clock.frames.x)
				.with_looped()
				.with_playing(),

			physics_timer: Timer::from_millis(2000),
			velocity: Point::default(),
			tween_progress: Tweenable::default(),
		}
	}

	fn update_physics(&mut self) {
		const DS: f32 = TitlesDisplay::SIZE;

		let spr = &mut self.sprite;

		self.velocity.x *= 0.98;
		self.velocity.y *= 0.98;

		self.velocity.y += 1.0;

		if spr.pos.x < 0.0 {
			spr.pos.x = 0.0;
			self.velocity.x *= -0.9;
		} else if spr.pos.x + spr.size.x > DS {
			spr.pos.x = DS - spr.size.x;
			self.velocity.x *= -0.9;
		}

		if spr.pos.y < 0.0 {
			spr.pos.y = 0.0;
			self.velocity.y *= -0.9;
		} else if spr.pos.y + spr.size.y > DS {
			spr.pos.y = DS - spr.size.y;
			self.velocity.y *= -0.9;
		}

		spr.pos.x += self.velocity.x;
		spr.pos.y += self.velocity.y;
	}

	fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		self.physics_timer.update(&ctx.time);
		self.tween_progress.update(&ctx.time);

		if self.sprite.is_hover(&mut ctx.input) && ctx.input.left_just_pressed() {
			self.physics_timer.start();
			self.velocity.x += quad_rand::gen_range(-20.0, 20.0);
			self.velocity.y += quad_rand::gen_range(-10.0, -20.0);
		}

		let spr = &mut self.sprite;

		if self.physics_timer.finished() {
			self.anim.frame = 0;

			if !self.tween_progress.playing() {
				self.tween_progress.play_from(
					0.0,
					1.0,
					Duration::from_millis(1000),
					Easing::Linear,
				);
			} else {
				let p = *self.tween_progress;

				spr.pos.x = spr.pos.x.lerp(Self::POS.x, p);
				spr.pos.y = spr.pos.y.lerp(Self::POS.y, p);
			}
		} else {
			self.anim.update(&ctx.time);
			self.update_physics();
		}

		draw_icon_text(
			ctx,
			canvas,
			Rect::new(Self::POS, self.sprite.size),
			b"clock",
		);

		self.sprite.frame.x = self.anim.frame;
		self.sprite.draw(&mut ctx.painter, canvas);
	}
}

/// Titles
pub struct Titles {
	clock: Clock,
}
impl Titles {
	pub fn new(ctx: &mut AppContext) -> Self {
		Self {
			clock: Clock::new(ctx),
		}
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId, titles_ctx: &mut TitlesContext) {
		// Draw background image
		Sprite::from(&ctx.assets.titles_bg).draw(&mut ctx.painter, canvas);

		// Draw title
		Text::new(&ctx.assets.serif_font)
			.with_pos((8.0, 8.0))
			.with_fg(Color::BLACK)
			.with_bg(Color::TRANSPARENT)
			.draw_chars(&mut ctx.painter, canvas, b"Thank you");

		self.draw_scoloc_icon(ctx, canvas, titles_ctx);

		self.clock.draw(ctx, canvas);
	}
	fn draw_scoloc_icon(
		&self,
		ctx: &mut AppContext,
		canvas: CanvasId,
		titles_ctx: &mut TitlesContext,
	) {
		const DS: f32 = TitlesDisplay::SIZE;
		const PADDING: f32 = 20.0;

		let mut sprite = Sprite::from(&ctx.assets.small_card).with_frame((9, 0));
		sprite.pos.set(PADDING, DS - sprite.size.y - PADDING - 8.0);
		sprite.draw(&mut ctx.painter, canvas);

		draw_icon_text(ctx, canvas, sprite.rect(), b"scoloc");

		if sprite.is_hover(&mut ctx.input) && ctx.input.left_just_pressed() {
			titles_ctx.goto_screen(Screen::Scoloc);
		}
	}
}
