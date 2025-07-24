use crate::app::Time;

/// Sound data
#[derive(Debug, Clone, Copy)]
pub struct SoundData(pub ());

#[macro_export]
macro_rules! include_sound_data {
	($file:expr) => {
		SoundData(())
	};
}

/// Sound
pub struct Sound;
impl Sound {
	pub fn play(&self) {
		/* no-op */
	}
	pub fn pause(&self) {
		/* no-op */
	}
	pub fn set_playing(&self, _play: bool) {
		/* no-op */
	}
	pub fn set_volume(&self, _volume: f32) {
		/* no-op */
	}
	pub fn is_playing(&self) -> bool {
		false
	}
}

/// Audio manager
pub struct Audio;
impl Audio {
	pub fn new(_muted: bool) -> Self {
		Self
	}
	pub fn new_sound<S>(&mut self, _source: S) -> Sound {
		Sound
	}
	pub fn new_sound_from_vorbis(&mut self, _data: SoundData, _looped: bool) -> Sound {
		Sound
	}

	pub fn play(&mut self, _data: SoundData) {
		/* no-op */
	}
	pub fn play_random(&mut self, _time: &Time, _datas: &[SoundData]) {
		/* no-op */
	}
}
