use std::time::Duration;

use crate::app::Time;

/// Timer
#[derive(Debug, Default, Clone)]
pub struct Timer {
	pub duration: Duration,
	/// Current time
	pub time: Duration,
}
impl Timer {
	pub fn new(duration: Duration) -> Self {
		Self {
			duration,
			time: Duration::default(),
		}
	}
	pub fn from_millis(millis: u64) -> Self {
		Self::new(Duration::from_millis(millis))
	}

	pub fn update(&mut self, time: &Time) {
		self.time = self.time.saturating_sub(time.delta);
	}
	pub fn start_duration(&mut self, duration: Duration) {
		self.duration = duration;
		self.time = duration;
	}
	pub fn start(&mut self) {
		self.time = self.duration;
	}

	pub fn progress(&self) -> f32 {
		if self.duration.is_zero() {
			return 1.0;
		}
		1.0 - self.time.as_millis() as f32 / self.duration.as_millis() as f32
	}
	pub fn left(&self) -> Duration {
		self.duration - self.time
	}
	pub fn finished(&self) -> bool {
		self.time.is_zero()
	}
}
