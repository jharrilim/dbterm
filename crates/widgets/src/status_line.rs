use ratatui::prelude::*;

#[derive(Debug)]
pub enum Status {
    Info(String),
    Error(String),
    Success(String),
    None,
}

#[derive(Debug)]
pub struct StatusLine {
    status: Option<Status>,
}

impl StatusLine {
    pub fn new() -> Self {
        Self { status: None }
    }

    pub fn set_text(&mut self, status: Status) {
        self.status = Some(status);
    }

    pub fn clear(&mut self) {
        self.status = None;
    }
}

impl Widget for &StatusLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1)].as_ref())
            .split(area);
        let bg = Color::Rgb(50, 50, 70);
        let text = match &self.status {
            Some(status) => match status {
                Status::Info(msg) => Text::raw(msg).style(Style::default().bg(bg).white()),
                Status::Error(msg) => Text::raw(msg).style(Style::default().bg(bg).red()),
                Status::Success(msg) => Text::raw(msg).style(Style::default().bg(bg).green()),
                Status::None => Text::raw("").style(Style::default().bg(bg).white()),
            },
            None => Text::raw("").style(Style::default().bg(bg).white()),
        };
        text.render(layout[0], buf);
    }
}
