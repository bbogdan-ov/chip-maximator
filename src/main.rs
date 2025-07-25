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

	let cli = Cli::new();

	miniquad::start(conf, move || Box::new(App::new(cli)));
}
