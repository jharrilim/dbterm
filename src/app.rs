use color_eyre::{eyre::Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
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

    /// Draw a single frame of the app.
    pub fn draw(&mut self) -> Result<()> {
        let app = &self.app;

        self.terminal
            .draw(|frame| app.render(frame.size(), frame.buffer_mut(), self.store.data()))
            .wrap_err("terminal.draw")?;
        Ok(())
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
            store,
            terminal,
            ..
        } = self;

        let (app_store_tx, app_store_rx) = tokio::sync::mpsc::unbounded_channel();
        let (render_tx, render_rx) = tokio::sync::mpsc::unbounded_channel::<RenderEvent>();


        let app = Arc::new(Mutex::new(app));
        crate::render::render_loop(terminal, app.clone(), store.data(), render_rx);

        Self::event_loop(app, store.data(), &app_store_tx);

        store.run(app_store_rx, render_tx).await;

        let mut stdout = io::stdout();
        term::restore(&mut stdout)?;
        Ok(())
    }
}

pub struct App {
    state: State,
    connection_screen: ConnectionScreen,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: State::ConnectionScreen,
            connection_screen: ConnectionScreen::new(),
        }
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
        }
        match self.state {
            State::ConnectionScreen => {
                return self.connection_screen.handle_event(event, ctx, tx);
            }
            State::MainScreen(_) => {}
        }
        Ok(false)
    }
}

impl AppWidget for App {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &Ctx) {
        match &self.state {
            State::ConnectionScreen => (&self.connection_screen).render(area, buf, ctx),
            State::MainScreen(tab) => {}
        };
    }
}
