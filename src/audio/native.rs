use std::mem;

use rodio::{
	OutputStream, OutputStreamBuilder, Sink, Source, buffer::SamplesBuffer, source::TriangleWave,
};

use crate::app::Time;

const SINKS_COUNT: usize = 16;

/// This structure fixes Rust's lack of custom alignment in `include_bytes!()`
/// Thanks to this blog post https://jack.wrenn.fyi/blog/include-transmute
#[repr(C)]
#[repr(align(4))] // align to f32
pub struct AlignedSoundBytes<T: ?Sized>(pub T);
impl AlignedSoundBytes<[u8]> {
	/// Convert `[u8]` to `[f32]`
	/// Function is taken from `bytemuck`:
	/// https://github.com/Lokathor/bytemuck/blob/0e11472150c3b63cbae3b445230fe074405bd2d2/src/internal.rs#L353
	#[inline]
	pub fn as_f32_slice(&self) -> &'static [f32] {
		// Check alignment
		assert_eq!((self.0.as_ptr() as usize) % mem::align_of::<f32>(), 0);

		let len = mem::size_of_val::<[u8]>(&self.0) / mem::size_of::<f32>();
		unsafe { core::slice::from_raw_parts(self.0.as_ptr() as *const f32, len) }
	}
}

/// Sound data
#[derive(Debug, Clone, Copy)]
pub struct SoundData(pub &'static [f32]);

#[macro_export]
macro_rules! include_sound_data {
	($file:expr) => {{
		use $crate::audio::AlignedSoundBytes as B;

		const DATA: &B<[u8]> = &B(*include_bytes!($file));
		$crate::audio::SoundData(DATA.as_f32_slice())
	}};
}

/// Sound
pub enum Sound {
	Normal { sink: Sink },
	Muted,
}
impl Sound {
	pub fn play(&self) {
		match self {
			Self::Normal { sink } => sink.play(),
			Self::Muted => (),
		}
	}
	pub fn pause(&self) {
		match self {
			Self::Normal { sink } => sink.pause(),
			Self::Muted => (),
		}
	}
	pub fn set_playing(&self, play: bool) {
		if !matches!(self, Self::Normal { .. }) {
			return;
		}

		if self.is_playing() != play {
			if play {
				self.play();
			} else {
				self.pause();
			}
		}
	}

	pub fn set_volume(&self, volume: f32) {
		match self {
			Self::Normal { sink } => sink.set_volume(volume),
			Self::Muted => (),
		}
	}

	pub fn is_playing(&self) -> bool {
		match self {
			Self::Normal { sink } => !sink.is_paused(),
			Self::Muted => false,
		}
	}
}

/// Audio manager
pub enum Audio {
	Normal {
		stream: OutputStream,
		sinks: [Sink; SINKS_COUNT],
	},
	Muted,
}
impl Audio {
	pub fn new(muted: bool) -> Self {
		if muted {
			Self::Muted
		} else {
			Self::new_normal()
		}
	}
	fn new_normal() -> Self {
		let stream = OutputStreamBuilder::open_default_stream().unwrap();

		// Populate N number of sinks
		let sinks = [(); SINKS_COUNT].map(|_| {
			let sink = Sink::connect_new(stream.mixer());
			sink.pause();
			sink
		});

		let buzz_sink = Sink::connect_new(stream.mixer());
		buzz_sink.append(TriangleWave::new(200.0).high_pass(500).amplify(0.2));
		buzz_sink.pause();

		Self::Normal { stream, sinks }
	}

	pub fn new_sound<S: Source + Send + 'static>(&mut self, source: S) -> Sound {
		let Self::Normal { stream, .. } = self else {
			return Sound::Muted;
		};

		let sink = Sink::connect_new(stream.mixer());
		sink.append(source);
		sink.pause();
		Sound::Normal { sink }
	}
	pub fn new_sound_from_vorbis(&mut self, data: SoundData, looped: bool) -> Sound {
		if !matches!(self, Self::Normal { .. }) {
			return Sound::Muted;
		};

		let buf = SamplesBuffer::new(1, super::SAMPLERATE, data.0);

		if looped {
			self.new_sound(buf.repeat_infinite())
		} else {
			self.new_sound(buf)
		}
	}

	pub fn play(&mut self, data: SoundData) {
		let Self::Normal { sinks, .. } = self else {
			return;
		};

		// Find the first empty sink
		let Some(sink) = sinks.iter().find(|s| s.empty()) else {
			return;
		};

		// Append and play the specified sound data
		// FIXME: i don't think this is the best practice to create a new `SamplesBuffer` on each
		//        sound playback...
		sink.append(SamplesBuffer::new(1, super::SAMPLERATE, data.0));
		sink.play();
	}
	pub fn play_random(&mut self, time: &Time, datas: &[SoundData]) {
		self.play(datas[time.elapsed as usize % datas.len()]);
	}
}
