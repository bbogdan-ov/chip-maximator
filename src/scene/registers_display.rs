use crate::{
	app::AppContext,
	math::Color,
	painter::{CanvasId, Text},
	state::State,
};

/// Front board registers display
pub struct RegistersDisplay {
	pub canvas: CanvasId,
}
impl RegistersDisplay {
	const SIZE: f32 = 8.0 * 4.0;
	const FG: Color = Color::new(0.7, 0.8, 0.7);
	const BG: Color = Color::gray(0.2);

	pub fn new(ctx: &mut AppContext) -> Self {
		Self {
			canvas: ctx.painter.context.new_canvas(
				(Self::SIZE, Self::SIZE),
				Self::BG,
				Default::default(),
			),
		}
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext, state: &mut State) {
		// Draw registers
		let mut text = Text::new(&ctx.assets.ibm_font)
			.with_fg(Self::FG)
			.with_bg(Color::TRANSPARENT);

		for i in 0..4_u8 {
			let x = i * 4;
			text.draw_line(&mut ctx.painter, self.canvas, &state.emu.regs[x..x + 4]);
		}
	}
}
