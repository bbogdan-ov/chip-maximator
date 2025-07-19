use crate::{
	app::AppContext,
	math::Point,
	painter::{CanvasId, Sprite},
	state::State,
};

/// Front board reset button
#[derive(Default)]
pub struct ResetButton;
impl ResetButton {
	pub fn draw(&self, ctx: &mut AppContext, state: &mut State, canvas: CanvasId) {
		const POS: Point = Point::new(457.0, 256.0);

		let mut sprite = Sprite::from(&ctx.assets.reset).with_pos(POS);

		let hovered = sprite.is_hover(&mut ctx.input);

		// Set tooltip
		if hovered {
			ctx.tooltip.set(b"Reset");
		}

		if hovered && ctx.input.left_is_pressed() {
			sprite.frame.x = 1;

			if ctx.input.left_just_pressed() {
				state.emu.setup();
				ctx.audio.play(ctx.assets.button_press_sound);
			}
		}

		sprite.draw(&mut ctx.painter, canvas);
	}
}
