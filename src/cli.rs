use std::{path::PathBuf, process::exit};

fn print_help() {
	println!("CHIP MAXIMATOR by bogdanov v{}", env!("CARGO_PKG_VERSION"));
	println!();
	println!("USAGE:");
	println!("    {bin} [options...] [rom]", bin = env!("CARGO_BIN_NAME"));
	println!();
	println!("OPTIONS:");
	println!("    --muted      Mute audio");
	println!("    --hello      Say \"hello\" to CHIP MAXIMATOR");
	println!("    -h, --help   Print this message");
}

/// Command line interface
#[derive(Default)]
pub struct Cli {
	pub muted: bool,
	pub rom_path: Option<PathBuf>,
}
impl Cli {
	pub fn parse(&mut self) {
		let mut args = std::env::args();
		args.next();

		for arg in args {
			if !arg.starts_with('-') {
				if self.rom_path.is_some() {
					eprintln!("ERROR: you can't specify more than one ROM files");
					exit(1);
				}

				self.rom_path = Some(PathBuf::from(arg));
				continue;
			}

			match arg.as_str() {
				"-h" | "--help" => {
					print_help();
					exit(0);
				}
				"--hello" => {
					println!("hi");
					exit(0);
				}

				"--muted" => self.muted = true,

				opt => {
					print_help();
					eprintln!("\nERROR: unknown option \"{opt}\"");
					exit(1);
				}
			}
		}
	}
}
