use std::sync::RwLock;

use ratatui::{buffer::Buffer, layout::Rect};

use crate::data::{Ctx, Data};

pub trait AppWidget {
  fn render(&self, area: Rect, buf: &mut Buffer, ctx: &Ctx);
}
