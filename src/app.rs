use clokwerk::{AsyncScheduler, TimeUnits};
use color_eyre::{eyre::Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use std::{
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};
use tokio::sync::mpsc::UnboundedSender;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::data::Data;
use crate::screens::connection::ConnectionScreen;
use crate::{events::EventHandler, term, IoEvent};

enum State {
    ConnectionScreen,
    MainScreen(MainScreenTabs),
}

enum MainScreenTabs {
    Querying,
    Monitoring,
}

pub struct Runtime {
    tx: Arc<Mutex<UnboundedSender<IoEvent>>>,
    scheduler_handle: Option<JoinHandle<()>>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    app: App,
}

impl Runtime {
    pub fn new(tx: UnboundedSender<IoEvent>, terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        let tx = Arc::new(Mutex::new(tx));
        Self {
            tx,
            scheduler_handle: None,
            terminal,
            app: App::new(),
        }
    }

    /// Draw a single frame of the app.
    pub fn draw(&mut self) -> Result<()> {
        let app = &self.app;

        self.terminal
            .draw(|frame| frame.render_widget(app, frame.size()))
            .wrap_err("terminal.draw")?;
        Ok(())
    }

    pub fn startup(&mut self) {}

    pub fn run_scheduler(&mut self) {
        let tx = self.tx.clone();
        let mut scheduler = AsyncScheduler::new();
        scheduler.every(60.seconds()).run(move || {
            let tx = tx.clone();
            async move {
                let _ = tx.lock().await.send(IoEvent::Test);
            }
        });

        self.scheduler_handle = Some(tokio::spawn(async move {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }));
    }

    pub fn stop_scheduler(&mut self) {
        self.scheduler_handle = None;
    }

    pub fn handle_event(&mut self) -> Result<bool> {
        let timeout = Duration::from_secs_f64(1.0 / 50.0);
        if let Some(e) = term::next_event(timeout)? {
            return self.app.handle_event(e);
        }
        Ok(false)
    }

    pub async fn run(&mut self) -> Result<()> {
        self.startup();
        // self.run_scheduler();
        let res = self.event_loop();
        self.stop_scheduler();
        let mut stdout = io::stdout();
        term::restore(&mut stdout)?;

        self.terminal.show_cursor()?;
        if let Err(err) = res {
            println!("{:?}", err)
        }
        Ok(())
    }

    fn event_loop(&mut self) -> Result<()> {
        loop {
            self.draw()?;

            if self.handle_event()? {
                return Ok(());
            }
        }
    }
}

struct App {
    state: State,
    connection_screen: ConnectionScreen,
    data: Data,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: State::ConnectionScreen,
            connection_screen: ConnectionScreen::new(),
            data: Data::new(),
        }
    }
}

impl EventHandler for App {
    fn handle_event(&mut self, event: Event) -> Result<bool> {
        if let Event::Key(key_event) = event {
            if let KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } = key_event
            {
                return Ok(true);
            }
        }
        match &mut self.state {
            State::ConnectionScreen => {
                if let Event::Key(key_event) = event {
                    if let KeyEvent {
                        code: KeyCode::Char('c'),
                        ..
                    } = key_event
                    {
                        self.state = State::MainScreen(MainScreenTabs::Querying);
                        return Ok(false);
                    }
                }

                return self.connection_screen.handle_event(event);
            }
            State::MainScreen(_) => {}
        }
        Ok(false)
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.state {
            State::ConnectionScreen => self.connection_screen.render(area, buf),
            State::MainScreen(tab) => {}
        };
    }
}
