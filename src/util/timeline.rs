use std::{fmt::Debug, rc::Rc, time::Duration};

use crate::{
	app::{AppContext, Time},
	audio::SoundData,
};

use super::{AnimRef, Easing, Timer, TweenableRef};

/// Animation waiting behaviour
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimWait {
	/// Jump to the next keyframe after the current animation has been finished
	Finish,
	/// Jump to the next keyframe after the current animation has reached the Nth frame
	Frame(i32),
}

/// Tweenable play descriptor
#[derive(Debug, Default, Clone, Copy)]
pub struct TweenPlay {
	pub end: f32,
	pub easing: Easing,
	pub duration: Duration,
}
impl TweenPlay {
	pub const fn new(end: f32, millis: u64, easing: Easing) -> Self {
		Self {
			end,
			easing,
			duration: Duration::from_millis(millis),
		}
	}
}

/// Tween action
#[derive(Debug, Clone, Copy)]
pub enum TweenAction {
	Play(TweenPlay),
	Set(f32),
}
impl From<TweenPlay> for TweenAction {
	fn from(value: TweenPlay) -> Self {
		Self::Play(value)
	}
}

/// Timeline keyframe
#[derive(Debug, Clone)]
pub enum Keyframe<A> {
	Action(A),
	Delay(Duration),
	Anim(AnimRef, AnimWait),
	Tween(TweenableRef, TweenAction),
	Sound(SoundData),
	Group(Box<[Keyframe<A>]>),
}
impl<A> Keyframe<A> {
	pub fn action(action: A) -> Self {
		Self::Action(action)
	}
	pub fn delay(millis: u64) -> Self {
		Self::Delay(Duration::from_millis(millis))
	}
	pub fn anim(anim: AnimRef, wait: AnimWait) -> Self {
		Self::Anim(anim, wait)
	}
	pub fn tween(tween: TweenableRef, action: impl Into<TweenAction>) -> Self {
		Self::Tween(tween, action.into())
	}
	pub fn sound(data: SoundData) -> Self {
		Self::Sound(data)
	}
	pub fn group(keyframes: impl Into<Box<[Keyframe<A>]>>) -> Self {
		Self::Group(keyframes.into())
	}
}

/// Timeline
#[derive(Debug)]
pub struct Timeline<A> {
	pub keyframes: Box<[Keyframe<A>]>,
	pub current: i32,
	pub delay_timer: Timer,
	pub playing: bool,
	pub reversed: bool,
	pub blocked: bool,

	pub cur_anim: Option<(AnimRef, AnimWait)>,
	pub cur_tween: Option<TweenableRef>,
}
impl<A: Copy> Timeline<A> {
	pub fn new(keyframes: impl Into<Box<[Keyframe<A>]>>) -> Self {
		Self {
			keyframes: keyframes.into(),
			current: 0,
			delay_timer: Timer::default(),
			playing: false,
			reversed: false,
			blocked: false,

			cur_anim: None,
			cur_tween: None,
		}
	}

	fn current_to_start(&mut self) {
		if self.reversed {
			self.current = self.keyframes.len() as i32 - 1;
		} else {
			self.current = 0;
		}
	}

	pub fn play(&mut self, reversed: bool) {
		if self.playing {
			return;
		}

		self.playing = true;
		self.reversed = reversed;
		self.current_to_start();
	}

	fn next_keyframe(&mut self) -> Option<&Keyframe<A>> {
		if self.current < 0 || self.current >= self.keyframes.len() as i32 {
			self.playing = false;
			return None;
		}

		let frame = &self.keyframes[self.current as usize];
		if self.reversed {
			self.current -= 1;
		} else {
			self.current += 1;
		}
		Some(frame)
	}
	/// FIXME: use reference (`&Keyframe<A>`) instead of owning the value
	fn execute_keyframe(&mut self, ctx: &mut AppContext, keyframe: Keyframe<A>) -> Option<A> {
		match keyframe {
			Keyframe::Action(action) => Some(action),
			Keyframe::Delay(delay) => {
				let delay = delay;
				self.delay_timer.start_duration(delay);
				None
			}
			Keyframe::Anim(anim, wait) => {
				anim.borrow_mut().play(self.reversed);
				self.cur_anim = Some((Rc::clone(&anim), wait));
				None
			}
			Keyframe::Tween(tween, action) => {
				let mut borrowed = tween.borrow_mut();
				match action {
					TweenAction::Play(play) => {
						borrowed.play(play.end, play.duration, play.easing);
						self.cur_tween = Some(Rc::clone(&tween));
					}
					TweenAction::Set(value) => {
						borrowed.value = value;
					}
				}

				None
			}
			Keyframe::Sound(data) => {
				ctx.audio.play(data);
				None
			}
			Keyframe::Group(s) => s
				.iter()
				.fold(None, |_, f| self.execute_keyframe(ctx, f.clone())),
		}
	}

	pub fn next(&mut self, ctx: &mut AppContext) -> Option<A> {
		if !self.playing || self.blocked {
			return None;
		}

		if self.is_waiting_delay() || self.is_waiting_tween() {
			return None;
		}
		if self.is_waiting_anim() {
			return None;
		}

		let frame = self.next_keyframe()?.clone();
		self.execute_keyframe(ctx, frame)
	}
	fn is_waiting_anim(&self) -> bool {
		let Some((anim, wait)) = self.cur_anim.as_ref() else {
			return false;
		};

		let anim = anim.borrow();
		match wait {
			AnimWait::Finish => anim.playing,
			AnimWait::Frame(frame) => anim.frame < *frame,
		}
	}
	fn is_waiting_tween(&self) -> bool {
		let Some(tween) = self.cur_tween.as_ref() else {
			return false;
		};

		tween.borrow().playing()
	}
	fn is_waiting_delay(&self) -> bool {
		!self.delay_timer.finished()
	}

	pub fn update(&mut self, time: &Time) {
		self.delay_timer.update(time);
	}
}
