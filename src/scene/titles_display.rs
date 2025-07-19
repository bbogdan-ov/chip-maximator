use crate::{
	app::AppContext,
	emu::Emu,
	math::{Color, ToStrBytes},
	painter::{CanvasId, Text, TextureOpts},
	state::State,
};

/// Back board titles display
pub struct TitlesDisplay {
	pub canvas: CanvasId,
}
impl TitlesDisplay {
	const SIZE: f32 = 256.0;

	pub fn new(ctx: &mut AppContext) -> Self {
		Self {
			canvas: ctx.painter.context.new_canvas(
				(Self::SIZE, Self::SIZE),
				Color::BLACK,
				TextureOpts {
					alpha: false,
					min_nearest: false,
					mag_nearest: false,
				},
			),
		}
	}

	pub fn offscreen_draw(&mut self, ctx: &mut AppContext, state: &mut State) {
		Text::new(&ctx.assets.serif_font)
			.with_font_size(0.7)
			.with_pos((8.0, 8.0))
			.with_bg(Color::TRANSPARENT)
			.draw_line(&mut ctx.painter, self.canvas, b"Hello guys");

		// let mut text = Text::new(&ctx.assets.ibm_font)
		// 	.with_font_size(2.0)
		// 	.with_pos((8.0, 8.0));
		//
		// text.draw_line(&mut ctx.painter, self.canvas, b"CHIP")
		// 	.draw_line(&mut ctx.painter, self.canvas, b"MAXIMATOR")
		// 	.new_line();
		//
		// for i in (0..20 * 2).step_by(2) {
		// 	let a = state.emu.program[i];
		// 	let b = state.emu.program[i + 1];
		// 	let ins = ((a as u16) << 8) | (b as u16);
		// 	let addr = (i + Emu::PROGRAM_START_ADDR) as u32;
		//
		// 	text.draw_chars(&mut ctx.painter, self.canvas, b"0x")
		// 		.draw_chars(
		// 			&mut ctx.painter,
		// 			self.canvas,
		// 			&addr.to_hex_str_bytes(false)[..3],
		// 		)
		// 		.draw_chars(&mut ctx.painter, self.canvas, b" - 0x")
		// 		.draw_chars(&mut ctx.painter, self.canvas, &ins.to_hex_str_bytes(true))
		// 		.new_line();
		// }
	}
}
