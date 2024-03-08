use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Widget};

pub struct Popup<'a> {
    title: &'a str,
}

impl<'a> Popup<'a> {
    pub fn new(title: &'a str) -> Self {
        Self { title }
    }

    fn layout(&self, area: Rect) -> Rect {
        Layout::default()
            .margin(1)
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area)[0]
    }

    pub fn render_body<B>(&self, area: Rect, buf: &mut Buffer, body: B)
    where
        B: FnOnce(Rect, &mut Buffer),
    {
        let area = self.layout(area);
        let area = centered_rect(80, 95, area);
        self.render(area, buf);
        body(area, buf);
    }
}

impl<'a> Widget for &Popup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().light_cyan());
        Clear.render(area, buf);
        block.render(area, buf);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
