use crate::{
	audio::SoundData,
	math::Point,
	painter::{CharWidth, Font, FontLookup, MAX_CHARS, Painter, Texture, TextureOpts},
};

/// Asset texture
pub struct AssetTexture {
	pub id: Texture,
	/// Size of each frame in the texture
	pub size: Point,
	/// Number of frames on each axis
	pub frames: Point<i32>,
}

macro_rules! assets {
	(
		textures {
			$($tex_name:ident => $tex_file_name:expr, $twidth:expr, $theight:expr, $frames_x:expr, $frames_y:expr),*$(,)?
		}
		fonts {
			$($font_name:ident => $font_file_name:expr, $fwidth:expr, $fheight:expr, $count:expr, $smooth:expr, $lookup:expr),*$(,)?
		}
		sounds {
			$($sound_name:ident => $sound_file_name:expr),*$(,)?
		}
	) => {
		/// Assets manager
		pub struct Assets {
			$(pub $tex_name: AssetTexture,)*
			$(pub $font_name: Font,)*
			$(pub $sound_name: SoundData,)*
		}
		impl Assets {
			pub fn new(painter: &mut Painter) -> Self {
				Self {
					// Load textures
					$($tex_name: {
						let bytes = include_bytes!(concat!(
							env!("OUT_DIR"),
							"/textures/",
							$tex_file_name,
							".png.bytes"
						));
						let texture = painter.context.new_texture(
							$twidth as i32 * $frames_x as i32,
							$theight as i32 * $frames_y as i32,
							Some(bytes),
							Default::default(),
						);

						AssetTexture {
							id: texture,
							size: ($twidth as f32, $theight as f32).into(),
							frames: ($frames_x as i32, $frames_y as i32).into(),
						}
					},)*

					// Load fonts
					$($font_name: {
						let bytes = include_bytes!(concat!(
							env!("OUT_DIR"),
							"/textures/",
							$font_file_name,
							".png.bytes"
						));
						let texture = painter.context.new_texture(
							$fwidth as i32 * $count as i32,
							$fheight as i32,
							Some(bytes),
							TextureOpts {
								alpha: true,
								min_nearest: !$smooth,
								mag_nearest: !$smooth,
							}
						);

						Font {
							texture,
							size: ($fwidth as f32, $fheight as f32).into(),
							count: $count as i32,
							lookup: $lookup,
						}
					},)*

					// Load sounds
					$($sound_name: crate::include_sound_data!(concat!(
						env!("OUT_DIR"),
						"/sounds/",
						$sound_file_name,
						".ogg.bytes"
					)),)*
				}
			}
		}
	};
}

fn serif_font_lookup() -> FontLookup {
	let upper_a_z: [u8; 26] = std::array::from_fn(|i| i as u8);
	let numbers: [u8; 10] = std::array::from_fn(|i| i as u8);

	let mut lookup = [0; MAX_CHARS];
	let mut widths = [CharWidth::Normal; MAX_CHARS];

	// Skip the first blank char
	let mut offset = 1_u8;

	// A-Z
	lookup[65..=90].copy_from_slice(&upper_a_z.map(|n| n + offset));
	offset += 26;
	// a-z
	lookup[97..=122].copy_from_slice(&upper_a_z.map(|n| n + offset));
	offset += 26;
	// 0-9
	lookup[48..=57].copy_from_slice(&numbers.map(|n| n + offset));
	offset += 10;
	// .
	lookup[46] = offset;
	offset += 1;
	// ,
	lookup[44] = offset;
	offset += 1;
	// -
	lookup[45] = offset;
	offset += 1;
	// !
	lookup[33] = offset;
	offset += 1;
	// ?
	lookup[63] = offset;

	for byte in b" iljft-,.!?".iter() {
		widths[*byte as usize] = CharWidth::Half;
	}
	for byte in b"IJsrpeao1".iter() {
		widths[*byte as usize] = CharWidth::ThreeQuarters;
	}

	FontLookup::Custom(lookup, widths)
}

assets! {
	textures {
		front_board => "front-board", 700, 700, 1, 1,
		board_flip => "board-flip", 700, 700, 10, 1,
		board_fall => "board-fall", 700, 700, 6, 1,
		back_board => "back-board", 700, 700, 6, 1,
		keyboard_key => "keyboard-key", 53, 54, 16, 2,
		led => "led", 14, 14, 16, 1,
		switch => "switch", 71, 41, 2, 1,
		valve => "valve", 128, 128, 6, 1,
		timers => "timers", 35, 123, 2, 1,
		state_led => "state-led", 11, 10, 4, 1,
		reset => "reset", 31, 31, 2, 1,
		heated_cpu => "heated-cpu", 74, 71, 1, 1,
		link => "link", 53, 53, 3, 2,
		slot => "slot", 126, 119, 2, 1,
		titles_bg => "titles-bg", 256, 256, 1, 1,
		card => "card", 71, 96, 13, 4,

		registers_display_uv => "registers-display-uv", 700, 700, 1, 1,
		game_display_uv => "game-display-uv", 700, 700, 1, 1,
		titles_display_uv => "titles-display-uv", 700, 700, 6, 1,
		movie_display_uv => "movie-display-uv", 700, 700, 6, 1,

		cartridge => "cartridge", 125, 133, 16, 1,

		explosion => "explosion", 175, 175, 16, 1,

		icons => "icons", 32, 32, 4, 4,
	}
	fonts {
		ibm_font => "ibm-font", 8, 8, 256, false, FontLookup::Ascii,
		serif_font => "serif-font", 26, 40, 72, true, {serif_font_lookup()},
	}
	sounds {
		// FIXME: key press/release sounds are kinda bad
		key_press_1_sound => "key-press-1",
		key_release_1_sound => "key-release-1",
		key_press_2_sound => "key-press-2",
		key_release_2_sound => "key-release-2",
		key_press_3_sound => "key-press-3",
		key_release_3_sound => "key-release-3",
		button_press_sound => "button-press",
		switch_toggle_sound => "switch-toggle",
		rotation_sound => "rotation",
		swipe_sound => "swipe",
		explosion_sound => "explosion",
		fan_sound => "fan",
		fall_sound => "fall",
		whistle_sound => "whistle",

		sound_0 => "0",
		sound_1 => "1",
		sound_2 => "2",
		sound_3 => "3",
		sound_4 => "4",
		sound_5 => "5",
		sound_6 => "6",
		sound_7 => "7",
		sound_8 => "8",
		sound_9 => "9",
		sound_a => "a",
		sound_b => "b",
		sound_c => "c",
		sound_d => "d",
		sound_e => "e",
		sound_f => "f",
	}
}
