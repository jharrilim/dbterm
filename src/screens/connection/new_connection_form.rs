use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Widget},
};
use tui_textarea::TextArea;

use crate::events::EventHandler;

use super::ConnectionInfo;

#[derive(Debug, Default)]
pub(crate) struct ConnectionInfoForm {
    name: TextArea<'static>,
    host: TextArea<'static>,
    port: TextArea<'static>,
    user: TextArea<'static>,
    password: TextArea<'static>,
    database: TextArea<'static>,
    state: ConnectionInfoFormState,
}

#[derive(Debug, PartialEq, PartialOrd, Default, Clone, Copy)]
enum ConnectionInfoFormState {
    #[default]
    Name,
    Host,
    Port,
    User,
    Password,
    Database,
}

impl ConnectionInfoForm {
    pub fn new() -> Self {
        let mut name = TextArea::default();
        name.set_block(
            Block::default()
                .title("Name")
                .borders(Borders::ALL)
                .magenta(),
        );

        let mut host = TextArea::default();
        host.set_block(Block::default().title("Host").borders(Borders::ALL));

        let mut port = TextArea::default();
        port.set_block(Block::default().title("Port").borders(Borders::ALL));

        let mut user = TextArea::default();
        user.set_block(Block::default().title("User").borders(Borders::ALL));

        let mut password = TextArea::default();
        password.set_mask_char('\u{2022}'); //U+2022 BULLET (•)
        password.set_block(Block::default().title("Password").borders(Borders::ALL));

        let mut database = TextArea::default();
        database.set_block(Block::default().title("Database").borders(Borders::ALL));

        Self {
            name,
            host,
            port,
            user,
            password,
            database,
            state: ConnectionInfoFormState::Name,
        }
    }

    fn inputs(&self) -> [&TextArea<'static>; 6] {
        [
            &self.name,
            &self.host,
            &self.port,
            &self.user,
            &self.password,
            &self.database,
        ]
    }

    fn inputs_mut(&mut self) -> [&mut TextArea<'static>; 6] {
        [
            &mut self.name,
            &mut self.host,
            &mut self.port,
            &mut self.user,
            &mut self.password,
            &mut self.database,
        ]
    }

    fn set_selected_input(&mut self) {
        let state = self.state;
        for (i, input) in self.inputs_mut().iter_mut().enumerate() {
            if i == (state as usize) {
                let block = input
                    .block()
                    .unwrap()
                    .clone()
                    .borders(Borders::ALL)
                    .magenta();
                input.set_style(Style::default());
                input.set_block(block);
            } else {
                input.set_style(Style::default());
                input.set_block(input.block().unwrap().clone().borders(Borders::ALL).gray());
            }
        }
    }

    pub fn to_connection_info(&self) -> ConnectionInfo {
        ConnectionInfo {
            name: self.name.lines()[0].clone(),
            host: self.host.lines()[0].clone(),
            port: self.port.lines()[0].parse().unwrap_or(5432),
            user: self.user.lines()[0].clone(),
            password: self.password.lines()[0].clone(),
            database: self.database.lines()[0].clone(),
        }
    }
}

impl EventHandler for ConnectionInfoForm {
    fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Tab => {
                        self.state = match self.state {
                            ConnectionInfoFormState::Name => ConnectionInfoFormState::Host,
                            ConnectionInfoFormState::Host => ConnectionInfoFormState::Port,
                            ConnectionInfoFormState::Port => ConnectionInfoFormState::User,
                            ConnectionInfoFormState::User => ConnectionInfoFormState::Password,
                            ConnectionInfoFormState::Password => ConnectionInfoFormState::Database,
                            ConnectionInfoFormState::Database => ConnectionInfoFormState::Name,
                        };
                        self.set_selected_input();
                        return Ok(false);
                    }
                    KeyCode::BackTab => {
                        self.state = match self.state {
                            ConnectionInfoFormState::Name => ConnectionInfoFormState::Database,
                            ConnectionInfoFormState::Host => ConnectionInfoFormState::Name,
                            ConnectionInfoFormState::Port => ConnectionInfoFormState::Host,
                            ConnectionInfoFormState::User => ConnectionInfoFormState::Port,
                            ConnectionInfoFormState::Password => ConnectionInfoFormState::User,
                            ConnectionInfoFormState::Database => ConnectionInfoFormState::Password,
                        };
                        self.set_selected_input();
                        return Ok(false);
                    }
                    _ => {
                        let state = self.state;
                        self.inputs_mut()[state as usize].input(event);
                    }
                }
            }
            _ => {}
        }

        Ok(false)
    }
}

impl Widget for &ConnectionInfoForm {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    // constraints for each input in the form
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(area);

        self.name.widget().render(layout[0], buf);

        let host_port_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Min(6)].as_ref())
            .split(layout[1]);
        self.host.widget().render(host_port_layout[0], buf);
        self.port.widget().render(host_port_layout[1], buf);

        let user_password_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)].as_ref())
            .split(layout[2]);
        self.user.widget().render(user_password_layout[0], buf);
        self.password.widget().render(user_password_layout[1], buf);

        self.database.widget().render(layout[3], buf);
    }
}

impl Widget for ConnectionInfoForm {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        (&self).render(area, buf)
    }
}
