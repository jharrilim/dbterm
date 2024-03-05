use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::widgets::Borders;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Widget},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::data::{AppCommand, Ctx};
use crate::events::EventHandler;
use crate::widget::AppWidget;

use super::ConnectionInfo;

#[derive(Debug)]
pub struct ConnectionList {
    selected: usize,
}

impl ConnectionList {
    pub fn new() -> Self {
        Self { selected: 0 }
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
            block.border_style(Style::default().fg(Color::LightMagenta))
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
    fn handle_event(
        &mut self,
        event: Event,
        ctx: &Ctx,
        _tx: &UnboundedSender<AppCommand>,
    ) -> Result<bool> {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else {
                        self.selected = ctx.read().unwrap().connections.len() - 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected < ctx.read().unwrap().connections.len() - 1 {
                        self.selected += 1;
                    } else {
                        self.selected = 0;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Ok(false)
    }
}

impl AppWidget for ConnectionList {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &Ctx)
    where
        Self: Sized,
    {
        let block = Block::default().title("Connections").borders(Borders::ALL);
        let block_area = block.inner(area);
        block.render(area, buf);

        let conns = &ctx.read().unwrap().connections;
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Start)
            .constraints::<Vec<Constraint>>(
                conns
                    .iter()
                    .map(|_| Constraint::Length(3))
                    .collect::<Vec<_>>(),
            )
            .split(block_area);

        for (i, connection) in conns.iter().enumerate() {
            self.render_connection_block(connection, self.selected == i, layout[i], buf);
        }
    }
}
