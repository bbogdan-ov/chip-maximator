use crate::{
	app::AppContext,
	input::InputConsume,
	state::{BoardSide, State},
	util::{Anim, AnimRef, AnimWait, Easing, Keyframe, Timeline, TweenPlay},
};

/// Board animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardAnim {
	Front,
	Back,
	Flipping,
	Falling,
}

/// Board timeline action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
	TogglePower(bool),
	SetAnim(BoardAnim),
	Reset,
}

/// Game
pub struct Game {
	cur_board_anim: BoardAnim,

	flip_timeline: Timeline<Action>,
	explode_timeline: Timeline<Action>,

	flip_anim: AnimRef,
	fall_anim: AnimRef,
	explosion_anim: AnimRef,
}
impl Game {
	pub fn new(ctx: &AppContext, state: &State) -> Self {
		let flip_anim = Anim::new(8, 0..ctx.assets.board_flip.frames.x).into_ref();
		let fall_anim = Anim::new(8, 0..ctx.assets.board_fall.frames.x).into_ref();
		let explosion_anim = Anim::new(16, 0..ctx.assets.explosion.frames.x).into_ref();

		let flip_timeline = Timeline::new([
			Keyframe::action(Action::TogglePower(true)),
			Keyframe::action(Action::TogglePower(false)),
			//
			Keyframe::tween(
				state.valve.angle.clone(),
				TweenPlay::new(0.0, 300, Easing::InOutSine),
			),
			//
			Keyframe::action(Action::SetAnim(BoardAnim::Front)),
			Keyframe::group([
				Keyframe::sound(ctx.assets.swipe_sound),
				Keyframe::anim(flip_anim.clone(), AnimWait::Finish),
				Keyframe::action(Action::SetAnim(BoardAnim::Flipping)),
			]),
			Keyframe::action(Action::SetAnim(BoardAnim::Back)),
		]);

		let explode_timeline = Timeline::new([
			Keyframe::sound(ctx.assets.explosion_sound),
			Keyframe::anim(explosion_anim.clone(), AnimWait::Frame(3)),
			//
			Keyframe::action(Action::Reset),
			Keyframe::action(Action::SetAnim(BoardAnim::Falling)),
			//
			Keyframe::delay(1500),
			Keyframe::anim(fall_anim.clone(), AnimWait::Finish),
			Keyframe::action(Action::SetAnim(BoardAnim::Front)),
		]);

		Self {
			cur_board_anim: match state.board.side {
				BoardSide::Front => BoardAnim::Front,
				BoardSide::Back => BoardAnim::Back,
			},

			flip_timeline,
			explode_timeline,

			flip_anim,
			fall_anim,
			explosion_anim,
		}
	}

	pub fn update(&mut self, ctx: &mut AppContext, state: &mut State) {
		// Consume input when any board anim is playing
		ctx.input.consume(
			InputConsume::BOARD_ANIM,
			self.flip_timeline.playing || self.explode_timeline.playing,
		);
	}
	fn update_anims(&mut self, ctx: &AppContext) {
		self.flip_anim.borrow_mut().update(&ctx.time);
		self.fall_anim.borrow_mut().update(&ctx.time);
		self.explosion_anim.borrow_mut().update(&ctx.time);
	}
	fn update_timelines(&mut self, ctx: &mut AppContext, state: &mut State) {
		self.flip_timeline.update(&ctx.time);
		self.explode_timeline.update(&ctx.time);

		while let Some(action) = self.flip_timeline.next(ctx) {
			self.handle_action(ctx, state, action);
		}
		while let Some(action) = self.explode_timeline.next(ctx) {
			self.handle_action(ctx, state, action);
		}
	}

	fn handle_action(&mut self, ctx: &mut AppContext, state: &mut State, action: Action) {
		match action {
			Action::TogglePower(power) => state.board.switch_power(ctx, power),
			Action::SetAnim(anim) => self.cur_board_anim = anim,
			Action::Reset => state.reset(),
		}
	}

	fn flip(&mut self, ctx: &mut AppContext, state: &State) {
		// Don't flip the board if CPU is hot
		if state.emu.is_hot() {
			ctx.tooltip.set_error(b"CPU is too hot!");
			return;
		}

		match self.cur_board_anim {
			BoardAnim::Front => self.flip_timeline.play(false),
			BoardAnim::Back => self.flip_timeline.play(true),
			BoardAnim::Flipping => (),
			BoardAnim::Falling => (),
		}
	}
	fn explode(&mut self) {
		self.explode_timeline.play(false);
	}
}
