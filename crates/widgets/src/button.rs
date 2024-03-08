use ratatui::{prelude::*, widgets::*};

#[derive(Debug)]
pub struct Button<'a> {
    text: &'a str,
    pub clicked: bool,
    pub highlighted: bool,
    border_style: Style,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            clicked: false,
            highlighted: false,
            border_style: Style::default(),
        }
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    pub fn click(&mut self) {
        self.clicked = !self.clicked;
    }

    pub fn highlight(&mut self) {
        self.highlighted = true;
    }

    pub fn unhighlight(&mut self) {
        self.highlighted = false;
    }
}

impl<'a> Widget for &Button<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.border_style);
        let block = if self.highlighted {
            block.border_style(Style::default().fg(Color::LightMagenta))
        } else {
            block
        };
        let block = if self.clicked {
            block.border_style(
                Style::default()
                    .fg(Color::LightMagenta)
                    .add_modifier(Modifier::REVERSED),
            )
        } else {
            block
        };
        let paragraph = Paragraph::new(self.text).block(block);

        paragraph.render(area, buf);
    }
}
