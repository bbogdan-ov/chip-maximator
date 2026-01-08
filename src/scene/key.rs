use miniquad::KeyCode;

use crate::{
	app::AppContext,
	assets::AssetTexture,
	painter::{CanvasId, Sprite},
};

/// Keyboard key sprite
pub struct Key {
	pub key_code: Option<KeyCode>,
	pub pressed: bool,
	pub just_pressed: bool,
	pub hovered: bool,
	sprite: Sprite,
}
impl Key {
	pub fn new(texture: &AssetTexture) -> Self {
		Self {
			key_code: None,
			pressed: false,
			just_pressed: false,
			hovered: false,
			sprite: Sprite::from(texture),
		}
	}

	pub fn set_frame(&mut self, frame: i32) {
		self.sprite.frame.set(frame, 0);
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		let input = &mut ctx.input;
		let (key_is_pressed, key_just_pressed, key_just_released) = match self.key_code {
			Some(key) => (
				input.key_is_pressed(key),
				input.key_just_pressed(key),
				input.key_just_released(key),
			),
			None => (false, false, false),
		};

		self.hovered = self.sprite.is_hover(input);
		self.pressed = self.hovered && input.left_is_pressed() || key_is_pressed;
		self.just_pressed = self.hovered && input.left_just_pressed() || key_just_pressed;
		let just_released = self.hovered && input.left_just_released() || key_just_released;

		// Set pressed frame
		if self.pressed {
			self.sprite.frame.y = 1;
		} else {
			self.sprite.frame.y = 0;
		}

		// Play press and release sounds
		if self.just_pressed {
			ctx.audio.play_random(
				&ctx.time,
				&[
					ctx.assets.key_press_1_sound,
					ctx.assets.key_press_2_sound,
					ctx.assets.key_press_3_sound,
				],
			);
		} else if just_released {
			ctx.audio.play_random(
				&ctx.time,
				&[
					ctx.assets.key_release_1_sound,
					ctx.assets.key_release_2_sound,
					ctx.assets.key_release_3_sound,
				],
			);
		}

		self.sprite.draw(&mut ctx.painter, canvas);
	}
}
impl std::ops::Deref for Key {
	type Target = Sprite;

	fn deref(&self) -> &Self::Target {
		&self.sprite
	}
}
impl std::ops::DerefMut for Key {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.sprite
	}
}
