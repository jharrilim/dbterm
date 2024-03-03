mod connection_list;
mod new_connection_form;

use new_connection_form::ConnectionInfoForm;

use crate::{data::ConnectionInfo, events::EventHandler, popup::Popup};
use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use serde::{Deserialize, Serialize};

use self::connection_list::ConnectionList;



#[derive(Debug, Default, PartialEq)]
enum State {
    #[default]
    Home,
    NewConnection,
}

#[derive(Debug)]
pub(crate) struct ConnectionScreen {
    connections: Vec<ConnectionInfo>,
    state: State,
    connections_list: ConnectionList,
    new_connection_form: ConnectionInfoForm,
}

impl ConnectionScreen {
    pub fn new() -> Self {
        let connections = ConnectionScreen::load_saved_connections();
        Self {
            connections,
            state: State::Home,
            new_connection_form: ConnectionInfoForm::new(),
            connections_list: ConnectionList::new(connections),
        }
    }

    fn connections_path() -> std::path::PathBuf {
        dirs::data_dir()
            .expect("No data dir")
            .join("dbterm")
            .join("connections.json")
    }

    fn load_saved_connections() -> Vec<ConnectionInfo> {
        let connections_path = ConnectionScreen::connections_path();
        if connections_path.exists() {
            std::fs::read_to_string(&connections_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
          vec![]
        }
    }

    fn save_connection(&mut self) -> Result<()> {
        let connection = self.new_connection_form.to_connection_info();

        let connections_path = ConnectionScreen::connections_path();
        if !connections_path.exists() {
            std::fs::create_dir_all(connections_path.parent().unwrap())?;
        }
        self.connections.push(connection);
        let content = serde_json::to_string(&self.connections)?;
        std::fs::write(&connections_path, content)?;
        Ok(())
    }

    
}

impl EventHandler for ConnectionScreen {
    fn handle_event(&mut self, event: Event) -> Result<bool> {
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
                }
                _ => return self.connections_list.handle_event(event),
            },
            State::NewConnection => {
                if event == Event::Key(KeyCode::Esc.into()) {
                    self.state = State::Home;
                    return Ok(false);
                }
                if let Event::Key(key_event) = event {
                    if key_event.code == KeyCode::Enter && key_event.kind == KeyEventKind::Press {
                        self.save_connection()?;
                        self.state = State::Home;
                        return Ok(false);
                    }
                }
                return self.new_connection_form.handle_event(event);
            }
        }
        Ok(false)
    }
}

impl Widget for &ConnectionScreen {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Max(2)].as_ref())
            .split(area);

        self.render_instructions(layout[1], buf);

        let connection_list = ConnectionList::new(self.connections);
        connection_list.render(layout[0], buf);

        match self.state {
            State::Home => {
            }
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
