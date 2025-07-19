use std::time::Duration;

unsafe extern "C" {
	pub fn performance_now() -> i32;
	pub fn window_open(url: *const i8, len: usize);
}

pub struct Instant {
	start: i32,
}
impl Instant {
	pub fn now() -> Self {
		Self {
			start: unsafe { performance_now() },
		}
	}

	pub fn elapsed(&self) -> Duration {
		unsafe {
			let ms = performance_now() - self.start;
			Duration::from_millis(ms as u64)
		}
	}
}
