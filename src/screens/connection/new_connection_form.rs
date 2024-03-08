use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use dbterm_widgets::{
    button::Button,
    picker::{Picker, PickerSelect},
    radio::RadioGroup,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{Block, Borders, Widget},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use crate::{
    data::{AppCommand, Ctx, DatabaseType, NewConnectionInfo},
    events::EventHandler,
};

use super::ConnectionInfo;

#[derive(Debug)]
pub(crate) struct ConnectionInfoForm {
    name: TextArea<'static>,
    host: TextArea<'static>,
    port: TextArea<'static>,
    user: TextArea<'static>,
    password: TextArea<'static>,
    database: TextArea<'static>,
    database_type: RadioGroup<'static, DatabaseType>,
    state: ConnectionInfoFormState,
    selected_database_type: Option<DatabaseType>,
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
    DatabaseTypePostgres,
    DatabaseTypeMysql,
    DatabaseTypeSqlite,
}

impl ConnectionInfoFormState {
    pub fn on_database_type(&self) -> bool {
        matches!(
            self,
            ConnectionInfoFormState::DatabaseTypePostgres
                | ConnectionInfoFormState::DatabaseTypeMysql
                | ConnectionInfoFormState::DatabaseTypeSqlite
        )
    }
}

impl ConnectionInfoForm {
    pub fn new() -> Self {
        let mut name = TextArea::default();
        name.set_block(
            Block::default()
                .title("Name")
                .borders(Borders::ALL)
                .light_magenta(),
        );

        let border_style = Style::default().white();

        let mut host = TextArea::default();
        host.set_block(
            Block::default()
                .title("Host")
                .borders(Borders::ALL)
                .border_style(border_style.clone()),
        );
        host.set_cursor_style(Style::default());

        let mut port = TextArea::default();
        port.set_block(
            Block::default()
                .title("Port")
                .borders(Borders::ALL)
                .border_style(border_style.clone()),
        );
        port.set_placeholder_text("5432");
        port.set_cursor_style(Style::default());

        let mut user = TextArea::default();
        user.set_block(
            Block::default()
                .title("User")
                .borders(Borders::ALL)
                .border_style(border_style.clone()),
        );
        user.set_cursor_style(Style::default());

        let mut password = TextArea::default();
        password.set_mask_char('\u{2022}'); //U+2022 BULLET (•)
        password.set_block(
            Block::default()
                .title("Password")
                .borders(Borders::ALL)
                .border_style(border_style.clone()),
        );
        password.set_cursor_style(Style::default());

        let mut database = TextArea::default();
        database.set_block(
            Block::default()
                .title("Database")
                .borders(Borders::ALL)
                .border_style(border_style.clone()),
        );
        database.set_cursor_style(Style::default());

        let database_type = RadioGroup::from(vec![
            ("Postgres", DatabaseType::Postgres),
            ("MySQL", DatabaseType::Mysql),
            ("SQLite", DatabaseType::Sqlite),
        ]);

        Self {
            name,
            host,
            port,
            user,
            password,
            database,
            database_type,
            state: ConnectionInfoFormState::Name,
            selected_database_type: None,
        }
    }

    #[allow(unused)]
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
                    .border_style(Style::default().light_magenta())
                    .light_magenta();
                input.set_style(Style::default());
                input.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
                input.set_block(block);
            } else {
                input.set_style(Style::default());
                input.set_cursor_style(Style::default());
                input.set_block(
                    input
                        .block()
                        .unwrap()
                        .clone()
                        .borders(Borders::ALL)
                        .border_style(Style::default().white())
                        .white(),
                );
            }
        }
        if self.state == ConnectionInfoFormState::DatabaseTypePostgres
            || self.state == ConnectionInfoFormState::DatabaseTypeMysql
            || self.state == ConnectionInfoFormState::DatabaseTypeSqlite
        {
            self.database_type.highlight(state as usize - self.inputs().len());
        } else {
            self.database_type.unhighlight();
        }
    }

    pub fn to_connection_info(&self) -> NewConnectionInfo {
        NewConnectionInfo {
            name: self.name.lines()[0].clone(),
            host: self.host.lines()[0].clone(),
            port: self.port.lines()[0].parse().unwrap_or(5432),
            user: self.user.lines()[0].clone(),
            password: self.password.lines()[0].clone(),
            database: self.database.lines()[0].clone(),
            database_type: DatabaseType::Postgres,
        }
    }
}

impl EventHandler for ConnectionInfoForm {
    fn handle_event(
        &mut self,
        event: Event,
        _ctx: &Ctx,
        _tx: &UnboundedSender<AppCommand>,
    ) -> Result<bool> {
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
                            ConnectionInfoFormState::Database => {
                                ConnectionInfoFormState::DatabaseTypePostgres
                            }
                            ConnectionInfoFormState::DatabaseTypePostgres => {
                                ConnectionInfoFormState::DatabaseTypeMysql
                            }
                            ConnectionInfoFormState::DatabaseTypeMysql => {
                                ConnectionInfoFormState::DatabaseTypeSqlite
                            }
                            ConnectionInfoFormState::DatabaseTypeSqlite => {
                                ConnectionInfoFormState::Name
                            }
                        };
                        self.set_selected_input();
                        return Ok(false);
                    }
                    KeyCode::BackTab => {
                        self.state = match self.state {
                            ConnectionInfoFormState::Name => {
                                ConnectionInfoFormState::DatabaseTypeSqlite
                            }
                            ConnectionInfoFormState::Host => ConnectionInfoFormState::Name,
                            ConnectionInfoFormState::Port => ConnectionInfoFormState::Host,
                            ConnectionInfoFormState::User => ConnectionInfoFormState::Port,
                            ConnectionInfoFormState::Password => ConnectionInfoFormState::User,
                            ConnectionInfoFormState::Database => ConnectionInfoFormState::Password,
                            ConnectionInfoFormState::DatabaseTypePostgres => {
                                ConnectionInfoFormState::Database
                            }
                            ConnectionInfoFormState::DatabaseTypeMysql => {
                                ConnectionInfoFormState::DatabaseTypePostgres
                            }
                            ConnectionInfoFormState::DatabaseTypeSqlite => {
                                ConnectionInfoFormState::DatabaseTypeMysql
                            }
                        };
                        self.set_selected_input();
                        return Ok(false);
                    }
                    KeyCode::Enter | KeyCode::Char(' ') if self.state.on_database_type() => {
                        self.database_type.select(self.state as usize - self.inputs().len());
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

        self.database_type.render(layout[4], buf);
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
