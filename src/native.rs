#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant as NativeInstant;

#[cfg(target_arch = "wasm32")]
pub use wasm::Instant as NativeInstant;

#[allow(unreachable_code)]
pub fn open_url(url: &str) -> Result<(), &'static str> {
	println!("Opening url {url:?}...");

	// Wasm
	#[cfg(target_arch = "wasm32")]
	unsafe {
		wasm::window_open(url.as_ptr() as _, url.len());
		return Ok(());
	}

	// Use `xdg-open` util on Linux
	// TODO: i don't know anything about other unix-like OS'es so only linux is supported for now
	#[cfg(target_os = "linux")]
	return Command::new("xdg-open")
		.arg(url)
		.spawn()
		.map_err(|_| "Unable to execute xdg-open")
		.map(|_| ());

	// Use `open` util on MacOS
	#[cfg(target_os = "macos")]
	return Command::new("open")
		.arg(url)
		.spawn()
		.map_err(|_| "Unable to execute open")
		.map(|_| ());

	// Use `start` util on Windows
	// FIXME: this should work, but i'm not sure...
	#[cfg(target_os = "windows")]
	return Command::new("start")
		.arg(url)
		.spawn()
		.map_err(|_| "Unable to execute start")
		.map(|_| ());

	// Any other unsupported target will return an error message
	Err("Your OS is't supported!!")
}
