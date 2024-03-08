use color_eyre::{eyre::Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use dbterm_widgets::status_line::{Status, StatusLine};
use ratatui::prelude::*;
use std::{
    io::{self, Stdout},
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::Mutex;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{
    data::{AppCommand, Ctx, Data, Store},
    screens::main::MainScreen,
    widget::AppWidget,
};
use crate::{events::EventHandler, term};
use crate::{render::RenderEvent, screens::connection::ConnectionScreen};

enum State {
    ConnectionScreen,
    MainScreen(MainScreenTabs),
}

enum MainScreenTabs {
    Querying,
    Monitoring,
}

pub struct Runtime {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    app: App,
    store: Store,
}

impl Runtime {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        Self {
            terminal,
            app: App::new(),
            store: Store::new(),
        }
    }

    pub fn event_loop(
        app: Arc<Mutex<App>>,
        data: &Arc<RwLock<Data>>,
        app_tx: &UnboundedSender<AppCommand>,
    ) -> JoinHandle<()> {
        let app = app.clone();
        let tx = app_tx.clone();
        let data = data.clone();
        tokio::spawn(async move {
            loop {
                let timeout = Duration::from_secs_f64(1.0 / 50.0);
                if let Ok(Some(e)) = term::next_event(timeout) {
                    if app
                        .clone()
                        .lock()
                        .await
                        .handle_event(e, &data, &tx)
                        .is_ok_and(|x| x)
                    {
                        tx.send(AppCommand::Quit).ok();
                        break;
                    }
                    tx.send(AppCommand::Render).ok();
                }
            }
        })
    }

    pub async fn run(self) -> Result<()> {
        let Self {
            app,
            mut store,
            terminal,
            ..
        } = self;

        let (app_store_tx, app_store_rx) = tokio::sync::mpsc::unbounded_channel();
        let (render_tx, render_rx) = tokio::sync::mpsc::unbounded_channel::<RenderEvent>();

        let app = Arc::new(Mutex::new(app));
        crate::render::render_loop(terminal, app.clone(), store.data(), render_rx);

        Self::event_loop(app, store.data(), &app_store_tx);

        store.run(app_store_rx, render_tx).await?;

        let mut stdout = io::stdout();
        term::restore(&mut stdout)?;
        Ok(())
    }
}

pub struct App {
    state: State,
    connection_screen: ConnectionScreen,
    main_screen: MainScreen,
    status_line: StatusLine,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            state: State::ConnectionScreen,
            connection_screen: ConnectionScreen::new(),
            main_screen: MainScreen::new(),
            status_line: StatusLine::new(),
        }
    }

    pub fn set_status_message(&mut self, message: Status) {
        self.status_line.set_text(message);
    }

    pub fn clear_status_message(&mut self) {
        self.status_line.clear();
    }

    pub fn goto_main_screen(&mut self) {
        self.state = State::MainScreen(MainScreenTabs::Querying);
    }

    pub fn set_query_result(&mut self, headers: Vec<String>, rows: Vec<Vec<String>>) {
        self.main_screen.set_output(headers, rows);
    }
}

impl EventHandler for App {
    fn handle_event(
        &mut self,
        event: Event,
        ctx: &Ctx,
        tx: &UnboundedSender<AppCommand>,
    ) -> Result<bool> {
        if let Event::Key(key_event) = event {
            if let KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } = key_event
            {
                return Ok(true);
            }
            // Needed or else Windows will trigger produce 2 events for each key press
            if key_event.kind != KeyEventKind::Press {
                return Ok(false);
            }
        }
        match self.state {
            State::ConnectionScreen => {
                return self.connection_screen.handle_event(event, ctx, tx);
            }
            State::MainScreen(_) => {
                return self.main_screen.handle_event(event, ctx, tx);
            }
        }
        Ok(false)
    }
}

impl AppWidget for App {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &Ctx) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(area);

        match &self.state {
            State::ConnectionScreen => (&self.connection_screen).render(layout[0], buf, ctx),
            State::MainScreen(_) => self.main_screen.render(layout[0], buf, ctx),
        };

        self.status_line.render(layout[1], buf);
    }
}
