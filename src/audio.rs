// TODO: may be there should be some sort of `AudioManager` and other audio controls traits...

/// Default sound samplerate
/// Every audio file should have this samplerate, otherwise it will sound wrong
pub const SAMPLERATE: u32 = 24_000;

// FIXME: audio is currently disabled in the web builds,
//        because i couldn't make `rodio` work on wasm32 builds...
#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
