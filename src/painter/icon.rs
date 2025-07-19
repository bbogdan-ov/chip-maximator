use crate::{app::AppContext, assets::Assets, math::Point};

use super::{CanvasId, Sprite};

/// Icon kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
	Flip,
	Pointer,
}
impl IconKind {
	pub fn into_frame(self) -> i32 {
		match self {
			Self::Flip => 0,
			Self::Pointer => 1,
		}
	}
}

/// Icon
pub struct Icon {
	inner: Sprite,
}
impl Icon {
	pub fn new(assets: &Assets, kind: IconKind) -> Self {
		let mut inner = Sprite::from(&assets.icons);
		inner.frame.x = kind.into_frame();

		Self { inner }
	}

	pub fn with_pos(mut self, pos: impl Into<Point>) -> Self {
		self.inner = self.inner.with_pos(pos);
		self
	}
	pub fn with_flip(mut self, flip: impl Into<Point<bool>>) -> Self {
		self.inner = self.inner.with_flip(flip);
		self
	}

	pub fn draw(&mut self, ctx: &mut AppContext, canvas: CanvasId) {
		self.inner.pos.x -= self.inner.size.x / 2.0;
		self.inner.pos.y -= self.inner.size.y / 2.0;

		self.inner.frame.y = ctx.icons_anim.frame;

		self.inner.draw(&mut ctx.painter, canvas);
	}
}
