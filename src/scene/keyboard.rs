use miniquad::KeyCode;

use crate::{app::AppContext, math::ToStrBytes, painter::CanvasId, scene::key::Key, state::State};

/// Converts CHIP-8 key to [`KeyCode`]
fn to_key_code(key: u8) -> KeyCode {
	[
		KeyCode::X,
		KeyCode::Key1,
		KeyCode::Key2,
		KeyCode::Key3,
		KeyCode::Q,
		KeyCode::W,
		KeyCode::E,
		KeyCode::A,
		KeyCode::S,
		KeyCode::D,
		KeyCode::Z,
		KeyCode::C,
		KeyCode::Key4,
		KeyCode::R,
		KeyCode::F,
		KeyCode::V,
	][key as usize]
}

/// Front board keyboard
#[derive(Default)]
pub struct Keyboard;
impl Keyboard {
	#[rustfmt::skip]
	const LAYOUT: &[i32] = &[
		0x1, 0x2, 0x3, 0xc,
		0x4, 0x5, 0x6, 0xd,
		0x7, 0x8, 0x9, 0xe,
		0xa, 0x0, 0xb, 0xf,
	];

	fn key_code_to_char(key: KeyCode) -> u8 {
		match key {
			KeyCode::Key1 => b'1',
			KeyCode::Key2 => b'2',
			KeyCode::Key3 => b'3',
			KeyCode::Key4 => b'4',

			KeyCode::Q => b'Q',
			KeyCode::W => b'W',
			KeyCode::E => b'E',
			KeyCode::R => b'R',

			KeyCode::A => b'A',
			KeyCode::S => b'S',
			KeyCode::D => b'D',
			KeyCode::F => b'F',

			KeyCode::Z => b'Z',
			KeyCode::X => b'X',
			KeyCode::C => b'C',
			KeyCode::V => b'V',

			_ => 0,
		}
	}

	pub fn draw(&self, ctx: &mut AppContext, state: &mut State, canvas: CanvasId) {
		// Keys position is sampled from GIMP
		const KEYBOARD_X: f32 = 131.0;
		const KEYBOARD_Y: f32 = 349.0;
		const KEYS_GAP_X: f32 = 53.0;
		const KEYS_GAP_Y: f32 = 57.0;

		let mut sprite = Key::new(&ctx.assets.keyboard_key);

		// Draw each key
		for (i, key) in Self::LAYOUT.iter().enumerate() {
			let col = i % 4;
			let row = i / 4;

			sprite.pos.set(
				KEYBOARD_X + KEYS_GAP_X * col as f32,
				KEYBOARD_Y + KEYS_GAP_Y * row as f32,
			);

			// Adjust position of some keys
			if row >= 2 {
				sprite.pos.y -= 1.0;
			}

			let key_code = to_key_code(*key as u8);

			sprite.key_code = Some(key_code);
			sprite.set_frame(*key);
			sprite.draw(ctx, canvas);

			// Show tooltip on hover
			if sprite.hovered {
				let mut msg = *b"Key 0x\0 [\0]";
				msg[6] = key.to_hex_str_bytes(false)[0];
				msg[9] = Self::key_code_to_char(key_code);
				ctx.tooltip.set(&msg);
			}

			state
				.emu
				.set_pressed_key(*key as u8, sprite.pressed, sprite.just_pressed);
		}
	}
}
