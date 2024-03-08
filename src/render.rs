use std::{
    io::Stdout,
    sync::{Arc, RwLock},
};

use color_eyre::eyre::Context;
use dbterm_widgets::status_line::Status;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::{
    sync::{mpsc::UnboundedReceiver, Mutex},
    task::JoinHandle,
};

use crate::{app::App, data::Data, widget::AppWidget};

pub enum RenderEvent {
    Draw,
    StatusMessage(Status),
    QueryResult {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Connected,
}

pub fn render_loop(
    mut term: Terminal<CrosstermBackend<Stdout>>,
    app: Arc<Mutex<App>>,
    data: &Arc<RwLock<Data>>,
    mut rx: UnboundedReceiver<RenderEvent>,
) -> JoinHandle<()> {
    let data = data.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let mut app = app.lock().await;
            macro_rules! draw {
                () => {
                    term.draw(|frame| app.render(frame.size(), frame.buffer_mut(), &data))
                        .wrap_err("terminal.draw")
                        .ok();
                };
            }
            match event {
                RenderEvent::Draw => {}
                RenderEvent::StatusMessage(status) => {
                    app.set_status_message(status);
                }
                RenderEvent::Connected => {
                    app.set_status_message(Status::Success("Connected".into()));
                    app.goto_main_screen();
                }
                RenderEvent::QueryResult { headers, rows } => {
                    app.set_query_result(headers, rows);
                }
            }
            draw!();
        }
        term.show_cursor().ok();
    })
}
