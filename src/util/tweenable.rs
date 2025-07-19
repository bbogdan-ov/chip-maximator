use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{app::Time, math::Lerp};

use super::Timer;

pub type TweenableRef = Rc<RefCell<Tweenable>>;

/// Easing function
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
	#[default]
	Linear,
	OutBack,
	OutCubic,
	InOutSine,
}
impl Easing {
	// These magic easing functions were taken from `easings.net`
	// Thanks!
	fn out_back(x: f32) -> f32 {
		const C1: f32 = 1.1;
		const C3: f32 = C1 + 1.5;

		1.0 + C3 * (x - 1.0).powi(3) + C1 * (x - 1.0).powi(2)
	}
	fn out_cubic(x: f32) -> f32 {
		1.0 - (1.0 - x).powi(3)
	}
	fn in_out_sine(x: f32) -> f32 {
		-((std::f32::consts::PI * x).cos() - 1.0) / 2.0
	}

	/// Calculate the result of the easing function
	/// `time` in range `0.0..=1.0` (but can overshot) and must be linear
	pub fn apply(&self, time: f32) -> f32 {
		match self {
			Self::Linear => time,
			Self::OutBack => Self::out_back(time),
			Self::OutCubic => Self::out_cubic(time),
			Self::InOutSine => Self::in_out_sine(time),
		}
	}
}

/// Tweenable float
/// Of cource i could use generics, but it increases code complexity when using [`crate::util::Timeline`]
/// Used for smooth interpolation between two values using ease function
#[derive(Debug, Clone)]
pub struct Tweenable {
	pub value: f32,

	start: f32,
	end: f32,
	easing: Easing,
	timer: Timer,
}
impl Default for Tweenable {
	fn default() -> Self {
		Self::new(0.0)
	}
}
impl Tweenable {
	pub fn new(default: f32) -> Self {
		Self {
			value: default,

			start: default,
			end: default,
			easing: Easing::Linear,
			timer: Timer::default(),
		}
	}

	pub fn play(&mut self, end: f32, duration: Duration, easing: Easing) {
		self.start = self.value;
		self.end = end;
		self.easing = easing;
		self.timer.start_duration(duration);
	}

	pub fn update(&mut self, time: &Time) {
		if !self.playing() {
			self.start = self.end;
			return;
		}

		self.timer.update(time);

		self.update_value();
	}
	fn update_value(&mut self) {
		let p = self.timer.progress();
		if p.abs() < f32::EPSILON {
			// Tween start
			self.value = self.start;
		} else if (p - 1.0).abs() < f32::EPSILON {
			// Tween end
			self.value = self.end;
		} else {
			// Tween is playing
			let a = self.easing.apply(p);
			self.value = self.start.lerp(self.end, a);
		}
	}

	pub fn into_ref(self) -> TweenableRef {
		Rc::new(RefCell::new(self))
	}

	pub fn playing(&self) -> bool {
		!self.timer.finished()
	}
}
impl std::ops::Deref for Tweenable {
	type Target = f32;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}
impl std::ops::DerefMut for Tweenable {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}
impl From<Tweenable> for TweenableRef {
	fn from(value: Tweenable) -> Self {
		value.into_ref()
	}
}
