use crate::{
	audio::SoundData,
	math::Point,
	painter::{Font, FontLookup, MAX_CHARS, Painter, START_CHAR, Texture, TextureOpts},
};

/// Asset texture
pub struct AssetTexture {
	pub id: Texture,
	/// Size of each frame in the texture
	pub size: Point,
	/// Number of frames on each axis
	pub frames: Point<i32>,
}

macro_rules! include_texture {
	($painter:expr, $path:expr, $width:expr, $height:expr, $opts:expr) => {{
		const SIZE: usize = $width as usize * $height as usize * 4;

		let bytes = include_bytes!($path);

		let decoded = lz4_flex::decompress(bytes, SIZE).expect("unable to decode texture data");
		$painter
			.context
			.new_texture($width as i32, $height as i32, Some(&decoded), $opts)
	}};
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
						let texture = include_texture!(
							painter,
							concat!(
								env!("OUT_DIR"),
								"/textures/",
								$tex_file_name,
								".png.bytes"
							),
							$twidth as i32 * $frames_x as i32,
							$theight as i32 * $frames_y as i32,
							Default::default()
						);

						if cfg!(debug_assertions) {
							println!("texture \"{}\" decompressed", stringify!($tex_name));
						}

						AssetTexture {
							id: texture,
							size: ($twidth as f32, $theight as f32).into(),
							frames: ($frames_x as i32, $frames_y as i32).into(),
						}
					},)*

					// Load fonts
					$($font_name: {
						let texture = include_texture!(
							painter,
							concat!(
								env!("OUT_DIR"),
								"/textures/",
								$font_file_name,
								".png.bytes"
							),
							$fwidth as i32 * $count as i32,
							$fheight as i32,
							TextureOpts {
								alpha: true,
								min_nearest: !$smooth,
								mag_nearest: !$smooth,
							}
						);

						if cfg!(debug_assertions) {
							println!("font \"{}\" decompressed", stringify!($font_name));
						}

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

pub const BASIC_FONT_LETTERS: &str =
	"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,-!?/#()";
pub const UPPER_ALPHA_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const LOWER_ALPHA_LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
pub const DIGIT_LETTERS: &str = "0123456789";
pub const SYMBOL_LETTERS: &str = ".,-!?/#()";

/// Basic indices lookup table for fonts with latin letters and some symbols
fn basic_font_lookup() -> [u8; MAX_CHARS] {
	let mut lookup = [0; MAX_CHARS];

	let mut offset: u8 = 1; // 1 to skip the first blank char

	macro_rules! make_lookup {
		($($start:expr, $end:expr;)*) => {
			$({
				let start = $start as usize - START_CHAR as usize;
				let end = $end as usize - START_CHAR as usize;
				for i in start..=end {
					lookup[i] = offset;
					offset += 1;
				}
			})*
		};
	}

	make_lookup! {
		'A', 'Z';
		'a', 'z';
		'0', '9';
		'.', '.';
		',', ',';
		'-', '-';
		'!', '!';
		'?', '?';
		'/', '/';
		'#', '#';
		'(', '(';
		')', ')';
	}

	lookup
}

macro_rules! make_widths {
	($($ch:expr, $width:expr;)*) => {{
		let mut widths: [u8; MAX_CHARS] = [10; MAX_CHARS];
		$(widths[$ch as usize - b' ' as usize] = $width;)*
		widths
	}};
}

fn serif_font_lookup() -> FontLookup {
	let widths = make_widths! {
		' ',10;
		'A',26; 'B',22; 'C',20; 'D',24; 'E',22; 'F',20; 'G',27; 'H',27; 'I',14; 'J',16; 'K',25; 'L',21; 'M',26; 'N',26; 'O',24; 'P',20; 'Q',24; 'R',24; 'S',18; 'T',22; 'U',26; 'V',25; 'W',25; 'X',24; 'Y',24; 'Z',20;
		'a',19; 'b',21; 'c',17; 'd',22; 'e',18; 'f',16; 'g',20; 'h',23; 'i',13; 'j',12; 'k',22; 'l',13; 'm',26; 'n',22; 'o',19; 'p',22; 'q',22; 'r',17; 's',16; 't',14; 'u',23; 'v',22; 'w',27; 'x',22; 'y',22; 'z',17;
		'0',20; '1',18; '2',19; '3',19; '4',22; '5',19; '6',20; '7',20; '8',20; '9',19;
		'.',8;  ';',11; '-',14; '!',9;  '?',17; '/',11; '#',19; '(',12; ')',12;
	};

	FontLookup::Custom {
		indices: basic_font_lookup(),
		widths,
	}
}
fn w98_font_lookup() -> FontLookup {
	let widths = make_widths! {
		' ',3;
		'A',8; 'B',6; 'C',7; 'D',7; 'E',6; 'F',6; 'G',7; 'H',7; 'I',2; 'J',5; 'K',7; 'L',6; 'M',8; 'N',7; 'O',7; 'P',7; 'Q',7; 'R',7; 'S',7; 'T',6; 'U',7; 'V',8; 'W',12; 'X',8; 'Y',8; 'Z',8;
		'a',6; 'b',6; 'c',6; 'd',6; 'e',6; 'f',3; 'g',6; 'h',6; 'i',2; 'j',2; 'k',6; 'l',2; 'm',8; 'n',6; 'o',6; 'p',6; 'q',6; 'r',3; 's',5; 't',3; 'u',6; 'v',6; 'w',8;  'x',5; 'y',6; 'z',5;
		'0',6; '1',4; '2',6; '3',6; '4',6; '5',6; '6',6; '7',6; '8',6; '9',6;
		'.',2; ';',3; '-',3; '!',2; '?',6; '/',5; '#',7; '(',3; ')',3;
	};

	FontLookup::Custom {
		indices: basic_font_lookup(),
		widths,
	}
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
		titles_bg => "titles-bg", 256, 256, 1, 1,
		card => "card", 71, 96, 13, 4,
		small_card => "small-card", 35, 51, 13, 1,
		clock => "clock", 50, 50, 8, 1,
		movie => "movie", 154, 128, 7, 1,

		registers_display_uv => "registers-display-uv", 700, 700, 1, 1,
		game_display_uv => "game-display-uv", 700, 700, 1, 1,
		titles_display_uv => "titles-display-uv", 700, 700, 6, 1,
		movie_display_uv => "movie-display-uv", 700, 700, 6, 1,

		cartridge => "cartridge", 125, 133, 16, 1,
		speaker => "speaker", 120, 190, 15, 1,
		speaker_window => "speaker-window", 274, 274, 1, 1,
		speaker_grid => "speaker-grid", 256, 128, 1, 1,
		speaker_button => "speaker-button", 24, 16, 2, 2,
		description_window => "description-window", 300, 300, 1, 1,
		description_diode => "description-diode", 26, 26, 1, 1,
		laptop_frame => "laptop-frame", 700, 700, 1, 1,

		explosion => "explosion", 175, 175, 16, 1,

		icons => "icons", 32, 32, 4, 4,
	}
	fonts {
		ibm_font => "ibm-font", 8, 8, 256, false, FontLookup::Monospace256,
		serif_font => "serif-font", 26, 40, 72, true, {serif_font_lookup()},
		w98_font => "w98-font", 12, 14, 72, false, {w98_font_lookup()},
	}
	sounds {
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
		grab_1_sound => "grab-1",
		grab_2_sound => "grab-2",
		grab_3_sound => "grab-3",
		grab_4_sound => "grab-4",
		suffering_1_sound => "suffering-1",
		suffering_2_sound => "suffering-2",
		suffering_3_sound => "suffering-3",

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
