mod connection_list;
mod new_connection_form;

use new_connection_form::ConnectionInfoForm;
use tokio::sync::mpsc::UnboundedSender;

use self::connection_list::ConnectionList;
use crate::{
    data::{AppCommand, ConnectionInfo, Ctx},
    events::EventHandler,
    popup::Popup,
    widget::AppWidget,
};
use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

#[derive(Debug, Default, PartialEq)]
enum State {
    #[default]
    Home,
    NewConnection,
}

#[derive(Debug)]
pub(crate) struct ConnectionScreen {
    state: State,
    connections_list: ConnectionList,
    new_connection_form: ConnectionInfoForm,
}

impl ConnectionScreen {
    pub fn new() -> Self {
        Self {
            state: State::Home,
            new_connection_form: ConnectionInfoForm::new(),
            connections_list: ConnectionList::new(),
        }
    }
}

impl EventHandler for ConnectionScreen {
    fn handle_event(
        &mut self,
        event: Event,
        ctx: &Ctx,
        tx: &UnboundedSender<AppCommand>,
    ) -> Result<bool> {
        match self.state {
            State::Home => match event {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Char('n')
                        && self.state == State::Home
                        && key_event.kind == KeyEventKind::Press
                    {
                        self.state = State::NewConnection;
                        self.new_connection_form = ConnectionInfoForm::new();
                        return Ok(false);
                    }
                    self.connections_list.handle_event(event, ctx, tx)
                }
                _ => self.connections_list.handle_event(event, ctx, tx),
            },
            State::NewConnection => {
                if event == Event::Key(KeyCode::Esc.into()) {
                    self.state = State::Home;
                    return Ok(false);
                }
                if let Event::Key(key_event) = event {
                    if key_event.code == KeyCode::Enter && key_event.kind == KeyEventKind::Press {
                        tx.send(AppCommand::SaveConnection(
                            self.new_connection_form.to_connection_info(),
                        ))
                        .ok();
                        self.state = State::Home;
                        return Ok(false);
                    }
                }
                self.new_connection_form.handle_event(event, ctx, tx)
            }
        }
    }
}

impl AppWidget for &ConnectionScreen {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &Ctx)
    where
        Self: Sized,
    {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Max(2)].as_ref())
            .split(area);

        self.render_instructions(layout[1], buf);

        self.connections_list.render(layout[0], buf, ctx);

        match self.state {
            State::Home => {}
            State::NewConnection => {
                let popup = Popup::new("New Connection");
                let form = &self.new_connection_form;
                popup.render_body(area, buf, |area, buf| {
                    form.render(area, buf);
                });
            }
        }
    }
}

impl ConnectionScreen {
    fn render_instructions(&self, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(
            "(n)ew connection, (e)dit connection, (d)elete connection, (c)onnect, (q)uit",
        )
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::TOP));
        paragraph.render(area, buf);
    }
}
