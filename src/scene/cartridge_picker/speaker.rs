use crate::{
	app::{AppContext, CANVAS_WIDTH},
	math::Point,
	painter::{CanvasId, Sprite},
};

pub struct Speaker {}
impl Speaker {
	const POS: Point = Point::new(CANVAS_WIDTH - 340.0, 30.0);

	pub fn new() -> Self {
		Self {}
	}

	pub fn draw(&self, ctx: &mut AppContext, canvas: CanvasId) {
		// Draw window frame
		let win_size = ctx.assets.speaker_window.size;
		Sprite::from(&ctx.assets.speaker_window)
			.with_pos(Self::POS)
			.draw(&mut ctx.painter, canvas);

		// Draw window buttons
		self.draw_buttons(ctx, canvas);

		// Draw speaker head
		let speaker_size = ctx.assets.speaker.size;
		Sprite::from(&ctx.assets.speaker)
			.with_pos((
				(Self::POS.x + win_size.x / 2.0 - speaker_size.x / 2.0).floor(),
				Self::POS.y + 78.0,
			))
			.draw(&mut ctx.painter, canvas);
	}
	fn draw_buttons(&self, ctx: &mut AppContext, canvas: CanvasId) {
		let play_pos = Self::POS + Point::new(26.0, 10.0);
		let stop_pos = Self::POS + Point::new(52.0, 10.0);

		let mut button = Sprite::from(&ctx.assets.speaker_button);

		button.frame.set(0, 0);
		button.pos = play_pos;
		if button.is_hover(&mut ctx.input) {
			if ctx.input.left_is_pressed() {
				button.frame.y = 1;
			}
		}
		button.draw(&mut ctx.painter, canvas);

		button.frame.set(1, 0);
		button.pos = stop_pos;
		if button.is_hover(&mut ctx.input) {
			if ctx.input.left_is_pressed() {
				button.frame.y = 1;
			}
		}
		button.draw(&mut ctx.painter, canvas);
	}
}
