use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::widgets::*;

#[derive(Debug)]
pub struct RadioGroup<'a, T> {
    radio_buttons: Vec<RadioButton<'a, T>>,
    selected: Option<usize>,
}

impl<'a, T> RadioGroup<'a, T> {
    pub fn selected(&self) -> Option<&T> {
        self.selected.map(|i| &self.radio_buttons[i].value)
    }
    pub fn select(&mut self, index: usize) {
        self.selected = Some(index);
        for (i, radio_button) in self.radio_buttons.iter_mut().enumerate() {
            radio_button.selected = i == index;
        }
    }

    pub fn highlight(&mut self, index: usize) {
        for (i, radio_button) in self.radio_buttons.iter_mut().enumerate() {
            if i == index {
                radio_button.highlight();
            } else {
                radio_button.unhighlight();
            }
        }
    }

    pub fn unhighlight(&mut self) {
        for radio_button in &mut self.radio_buttons {
            radio_button.unhighlight();
        }
    }
}

impl<'a, T> Widget for &RadioGroup<'a, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .flex(Flex::Start)
            .constraints(
                self.radio_buttons
                    .iter()
                    .map(|radio_button| Constraint::Length(radio_button.size() as u16))
                    .collect::<Vec<_>>(),
            )
            .split(area);

        for (i, radio_button) in self.radio_buttons.iter().enumerate() {
            radio_button.render(layout[i], buf);
        }
    }
}

impl<'a, T> From<Vec<(&'a str, T)>> for RadioGroup<'a, T> {
    fn from(radio_buttons: Vec<(&'a str, T)>) -> Self {
        let radio_buttons = radio_buttons
            .into_iter()
            .map(|(text, value)| RadioButton {
                text,
                value,
                selected: false,
                highlighted: false,
            })
            .collect();
        Self {
            radio_buttons,
            selected: None,
        }
    }
}

#[derive(Debug)]
pub struct RadioButton<'a, T> {
    text: &'a str,
    value: T,
    selected: bool,
    highlighted: bool,
}

impl<'a, T> RadioButton<'a, T> {
    pub fn size(&self) -> usize {
        self.text.len() + 2 // for the bullet and the space
    }
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn highlight(&mut self) {
        self.highlighted = true;
    }

    pub fn unhighlight(&mut self) {
        self.highlighted = false;
    }
}

impl<'a, T> Widget for &RadioButton<'a, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = if self.selected {
            format!("◉ {}", self.text)
        } else {
            format!("◎ {}", self.text)
        };
        let paragraph = Paragraph::new(text);
        let paragraph = if self.highlighted {
          paragraph.magenta()
        } else {
          paragraph.gray()
        };
        paragraph.render(area, buf);
    }
}
