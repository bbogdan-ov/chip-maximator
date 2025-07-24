use std::{fs, path::PathBuf};

use png::{BitDepth, ColorType};

fn main() {
	println!("cargo::rerun-if-changed=assets/textures");
	println!("cargo::rerun-if-changed=assets/sounds");

	decode_textures();
	decode_sounds();
}

/// Creates dir inside OUT_DIR if not exists
fn create_out_subdir(name: &str) -> PathBuf {
	let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	let dir = out_dir.join(name);
	if !fs::exists(&dir).unwrap() {
		fs::create_dir(dir.clone()).unwrap();
	}

	dir
}

/// Predecode all texture files in `assets/textures` dir,
/// so we can include them into executable via `include_bytes!()` later
fn decode_textures() {
	let out_dir = create_out_subdir("textures");

	// Iterate through all texture files in the textures dir
	let read_dir = fs::read_dir("assets/textures").unwrap();
	for entry in read_dir {
		let entry = entry.unwrap();
		if !entry.file_type().unwrap().is_file() {
			continue;
		}

		// Read file
		let path = entry.path();
		let file = fs::File::open(path.clone()).unwrap();

		// Decode png data
		let decoder = png::Decoder::new(file);
		let mut info = decoder.read_info().unwrap();
		let color_type = info.output_color_type();
		assert_eq!(
			color_type.0,
			ColorType::Rgba,
			"Must be RGBA color type at {path:?}"
		);
		assert_eq!(
			color_type.1,
			BitDepth::Eight,
			"Must be 8-bit color at {path:?}"
		);

		// Write pixel data into buffer
		let mut buf = vec![0_u8; info.output_buffer_size()];
		info.next_frame(&mut buf).unwrap();

		// Compress and store pixel data into a file
		let compressed = lz4_flex::compress(&buf);
		let filename = entry.file_name();
		let filename = format!("{}.bytes", filename.to_string_lossy());
		let path = out_dir.clone().join(filename);
		fs::write(path, &compressed).unwrap();
	}
}

fn decode_sounds() {
	let out_dir = create_out_subdir("sounds");

	// Iterate through all sound files in the sounds dir
	let read_dir = fs::read_dir("assets/sounds").unwrap();
	for entry in read_dir {
		let entry = entry.unwrap();
		if !entry.file_type().unwrap().is_file() {
			continue;
		}

		// Read file
		let path = entry.path();
		let file = fs::File::open(path.clone()).unwrap();

		// Decode audio data
		let decoder = rodio::Decoder::new_vorbis(file).unwrap();
		let buf = decoder.flat_map(|s| s.to_ne_bytes()).collect::<Vec<u8>>();

		// Store sample data into a file
		let filename = entry.file_name();
		let filename = format!("{}.bytes", filename.to_string_lossy());
		let path = out_dir.clone().join(filename);
		fs::write(path, &buf).unwrap();
	}
}
