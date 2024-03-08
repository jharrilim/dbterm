use ratatui::{buffer::Buffer, layout::Rect};

use crate::data::Ctx;

pub trait AppWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &Ctx);
}
