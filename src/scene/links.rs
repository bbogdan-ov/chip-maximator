use crate::{app::AppContext, native, painter::CanvasId, scene::key::Key};

/// Back board links keyboard
#[derive(Default)]
pub struct Links;
impl Links {
	#[rustfmt::skip]
	const LINKS: &[(&str, &str)] = &[
		("My website! - bbogdan-ov.github.io", "https://bbogdan-ov.github.io"),
		("My telegram - @bbogdan_ov", "https://t.me/bbogdan_ov"),
		("And github - @bbogdan-ov", "https://github.com/bbogdan-ov"),
	];

	pub fn draw(&self, ctx: &mut AppContext, canvas: CanvasId) {
		const KEYBOARD_X: f32 = 363.0;
		const KEYBOARD_Y: f32 = 529.0;
		const KEYS_GAP_X: f32 = 53.0;

		let mut sprite = Key::new(&ctx.assets.link);

		for (i, (name, link)) in Self::LINKS.iter().enumerate() {
			sprite.set_frame(i as i32);
			sprite
				.pos
				.set(KEYBOARD_X + KEYS_GAP_X * i as f32, KEYBOARD_Y);

			sprite.draw(ctx, canvas);

			// Set tooltip
			if sprite.hovered {
				ctx.tooltip.set(name.as_bytes());
			}

			// Open the URL on press
			if sprite.just_pressed {
				let res = native::open_url(link);
				// Show message if an error has occured
				if let Err(e) = res {
					ctx.tooltip.set_error(e.as_bytes());
				}
			}
		}
	}
}
