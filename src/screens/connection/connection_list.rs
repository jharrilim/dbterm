use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::widgets::Borders;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Widget},
};

use crate::events::EventHandler;

use super::ConnectionInfo;

#[derive(Debug)]
pub struct ConnectionList {
    connections: Vec<ConnectionInfo>,
    selected: usize,
}

impl ConnectionList {
    pub fn new(connections: Vec<ConnectionInfo>) -> Self {
        Self {
            connections,
            selected: 0,
        }
    }

    // renders a block for a single connection showcasing all of its info
    pub fn render_connection_block(
        &self,
        connection: &ConnectionInfo,
        selected: bool,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let block = Block::default()
            .title(&*connection.name)
            .borders(Borders::ALL);

        let block = if selected {
            block.border_style(Style::default().fg(Color::Yellow))
        } else {
            block
        };

        let block_area = block.inner(area);
        block.render(area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1)].as_ref())
            .split(block_area);

        let host_and_port_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Min(1)].as_ref())
            .split(layout[0]);
        let text = format!("Host: {}", connection.host);
        let text = Text::raw(text);
        text.render(host_and_port_layout[0], buf);
        let text = format!("Port: {}", connection.port);
        let text = Text::raw(text);
        text.render(host_and_port_layout[1], buf);
    }
}

impl EventHandler for ConnectionList {
    fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.selected < self.connections.len() - 1 {
                        self.selected += 1;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Ok(false)
    }
}

impl Widget for &ConnectionList {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::default().title("Connections").borders(Borders::ALL);
        let block_area = block.inner(area);
        block.render(area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Start)
            .constraints::<Vec<Constraint>>(
                self.connections
                    .iter()
                    .map(|_| Constraint::Length(3))
                    .collect::<Vec<_>>(),
            )
            .split(block_area);

        for (i, connection) in self.connections.iter().enumerate() {
            self.render_connection_block(connection, self.selected == i, layout[i], buf);
        }
    }
}
