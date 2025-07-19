macro_rules! game {
	($name:expr) => {
		GameInfo {
			title: concat!($name, ".ch8"),
			desc: include_str!(concat!("../roms/", $name, ".txt")),
			bytes: include_bytes!(concat!("../roms/", $name, ".ch8")),
		}
	};
}

/// Game info
#[derive(Debug)]
pub struct GameInfo {
	/// Game name
	pub title: &'static str,
	/// Game description
	pub desc: &'static str,
	/// Game program data
	pub bytes: &'static [u8],
}

pub const GAMES: &[GameInfo] = &[
	game!("tracer"),
	game!("langtons-ant"),
	game!("outlaw"),
	game!("vers"),
	game!("rush-hour"),
	game!("blinky"),
	game!("worm-v4"),
	game!("tic-tac-toe"),
	game!("breakout"),
	game!("landing"),
	game!("puzzle"),
	game!("space-invaders"),
	game!("pong"),
];
