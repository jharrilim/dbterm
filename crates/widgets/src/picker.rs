use crate::popup::Popup;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List},
};

#[derive(Debug)]
pub struct Picker<'a, 'b, I> {
    title: &'a str,
    items: Vec<(&'b str, I)>,
    selected: Option<Vec<usize>>,
    index: usize,
    select: PickerSelect,
}

#[derive(Debug)]
pub enum PickerSelect {
    Single,
    Multiple,
}

impl<'a, 'b, I> Picker<'a, 'b, I> {
    pub fn new(title: &'a str, items: Vec<(&'b str, I)>, select: PickerSelect) -> Self {
        Self {
            title,
            items,
            index: 0,
            selected: None,
            select,
        }
    }

    pub fn single(title: &'a str, items: Vec<(&'b str, I)>) -> Self {
        Self::new(title, items, PickerSelect::Single)
    }

    pub fn multiple(title: &'a str, items: Vec<(&'b str, I)>) -> Self {
        Self::new(title, items, PickerSelect::Multiple)
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.items.len();
    }

    pub fn previous(&mut self) {
        if self.index == 0 {
            self.index = self.items.len() - 1;
        } else {
            self.index -= 1;
        }
    }

    pub fn select(&mut self) {
        if let Some(selected) = &mut self.selected {
            if selected.contains(&self.index) {
                selected.retain(|&x| x != self.index);
            } else {
                match self.select {
                    PickerSelect::Single => {
                        selected.clear();
                        selected.push(self.index);
                    }
                    PickerSelect::Multiple => {
                        selected.push(self.index);
                    }
                }
            }
        } else {
            self.selected = Some(vec![self.index]);
        }
    }

    pub fn selected(&self) -> Option<Vec<&I>> {
        self.selected.as_ref().map(|selected| {
            selected
                .iter()
                .map(|&index| &self.items[index].1)
                .collect()
        })
    }
}

impl<'a, 'b, I> Widget for &Picker<'a, 'b, I> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items = self.items.iter().map(|(item, _)| item).cloned();
        let list = List::new(items)
            .block(Block::default().title(self.title).borders(Borders::ALL))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>");

        let popup = Popup::new(self.title);

        popup.render_body(area, buf, |area, buf| {
            Widget::render(&list, area, buf);
        });
    }
}
