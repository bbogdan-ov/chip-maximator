mod app;
mod assets;
mod audio;
mod cli;
mod emu;
mod games;
mod input;
mod math;
mod native;
mod painter;
mod scene;
mod state;
mod tooltip;
mod util;

use app::App;
use cli::Cli;
use miniquad::conf;

fn main() {
	let conf = conf::Conf {
		window_title: "CHIP MAXIMATOR - by bogdanov".to_string(),
		window_width: 700,
		window_height: 700,
		platform: conf::Platform {
			// Use X11 first because previously miniquad was crashing every time it was launched on
			// Wayland compositor (at least on Hyprland) with Wayland first backend.
			// I'm afraid it's still crashing on some Wayland compositors...
			linux_backend: conf::LinuxBackend::X11WithWaylandFallback,
			linux_wm_class: "chip-maximator",
			webgl_version: conf::WebGLVersion::WebGL2,
			..Default::default()
		},
		..Default::default()
	};

	let mut cli = Cli::default();
	cli.parse();

	// Set random seed
	quad_rand::srand(native::now_millis() as u64);

	miniquad::start(conf, move || {
		// Register custom panic hook here, because miniquad registers it in the `start`
		// function so we need to override it. Also miniquad's panic message says
		// fucking nothing about the error itself
		set_hook();
		Box::new(App::new(cli))
	});
}

fn set_hook() {
	#[cfg(target_arch = "wasm32")]
	unsafe {
		use miniquad::native::wasm::console_log;
		use std::ffi::CString;

		std::panic::set_hook(Box::new(|info| {
			if let Some(loc) = info.location() {
				let loc_str = CString::new(format!(
					"in file {:?} at {}:{}",
					loc.file(),
					loc.line(),
					loc.column()
				))
				.unwrap();

				console_log(loc_str.as_ptr());
			} else {
				console_log(c"NO LOCATION".as_ptr());
			}

			let payload: CString;
			if let Some(s) = info.payload().downcast_ref::<&str>() {
				payload = CString::new(*s).unwrap();
			} else if let Some(s) = info.payload().downcast_ref::<String>() {
				payload = CString::new(s.clone()).unwrap();
			} else {
				payload = CString::new("NO PAYLOAD").unwrap();
			}

			console_log(payload.as_ptr());
		}));
	}
}
