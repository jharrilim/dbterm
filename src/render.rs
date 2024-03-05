use std::{
    io::Stdout,
    sync::{Arc, RwLock},
};

use color_eyre::eyre::Context;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::{sync::{mpsc::UnboundedReceiver, Mutex}, task::JoinHandle};

use crate::{app::App, data::Data, widget::AppWidget};

pub enum RenderEvent {
    Draw,
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
            let app = app.lock().await;
            match event {
                RenderEvent::Draw => {
                    term.draw(|frame| app.render(frame.size(), frame.buffer_mut(), &data))
                        .wrap_err("terminal.draw")
                        .ok();
                }
            }
        }
        term.show_cursor().ok();
    })
}
