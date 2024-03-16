use crate::data::AppCommand;
use crate::data::Ctx;
use crate::events::EventHandler;
use crate::widget::AppWidget;
use color_eyre::eyre::Result;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::prelude::*;
use ratatui::widgets::*;
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

enum State {
    Query,
}

pub struct MainScreen {
    output: TextArea<'static>,
    input: TextArea<'static>,
    state: State,
}

impl MainScreen {
    pub fn new() -> Self {
        let body = Block::default()
            .title("Results")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightMagenta));

        let mut output = TextArea::default();
        output.set_line_number_style(Style::default());
        output.set_block(body);

        let footer = Block::default()
            .title("Query")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightMagenta));

        let mut input = TextArea::default();
        input.set_block(footer);

        Self {
            output,
            input,
            state: State::Query,
        }
    }

    pub fn set_output(&mut self, headers: Vec<String>, rows: Vec<Vec<String>>) {
        let body = Block::default()
            .title("Results")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightMagenta));

        let rows = rows
            .iter()
            .map(|row| row.join("\t"))
            .collect::<Vec<String>>();

        let rows = vec![headers.join("\t")].into_iter().chain(rows).collect::<Vec<_>>();
        let mut output = TextArea::from(rows);
        output.set_line_number_style(Style::default());
        output.set_block(body);
        self.output = output;
    }
}

impl EventHandler for MainScreen {
    fn handle_event(
        &mut self,
        event: Event,
        _ctx: &Ctx,
        tx: &UnboundedSender<AppCommand>,
    ) -> Result<bool> {

        match self.state {
            State::Query => match event {
                Event::Key(key_event) => match key_event {
                    KeyEvent {
                        code: KeyCode::Enter,
                        // switch to cmd/ctrl + enter after this is resolved https://github.com/crossterm-rs/crossterm/issues/515
                        ..
                    } => {
                        tx.send(AppCommand::Query(self.input.lines().join("\n")))
                            .ok();
                    }
                    _ => {
                        self.input.input(event);
                    }
                },
                _ => {
                    self.input.input(event);
                }
            },
        };
        Ok(false)
    }
}

impl AppWidget for MainScreen {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &Ctx) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(area);

        let header = Block::default()
            .title("Header")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightMagenta));
        header.render(layout[0], buf);
        self.output.widget().render(layout[1], buf);
        self.input.widget().render(layout[2], buf);
    }
}
