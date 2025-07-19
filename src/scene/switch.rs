use crate::{
	app::AppContext,
	math::Point,
	painter::{CanvasId, Sprite},
	state::State,
};

/// Front board switch
/// Used to turn power on/off
#[derive(Default)]
pub struct Switch;
impl Switch {
	pub fn draw(&self, ctx: &mut AppContext, state: &mut State, canvas: CanvasId) {
		const POS: Point = Point::new(455.0, 163.0);

		let mut sprite = Sprite::from(&ctx.assets.switch).with_pos(POS);

		if sprite.is_hover(&mut ctx.input) {
			ctx.tooltip.set(b"Switch");

			if ctx.input.left_just_pressed() {
				state.board.toggle_power(ctx);
			}
		}

		sprite.frame.x = state.board.power as i32;
		sprite.draw(&mut ctx.painter, canvas);
	}
}
