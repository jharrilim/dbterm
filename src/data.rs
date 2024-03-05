use std::sync::{Arc, RwLock};

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::render::RenderEvent;

pub type Ctx = RwLock<Data>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Data {
    pub connections: Vec<ConnectionInfo>,
}

impl Data {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save_connection(&mut self, connection: ConnectionInfo) -> Result<()> {
        let connections_path = connections_path();
        if !connections_path.exists() {
            std::fs::create_dir_all(connections_path.parent().unwrap())?;
        }
        self.connections.push(connection);
        let content = serde_json::to_string(&self.connections)?;
        std::fs::write(&connections_path, content)?;
        Ok(())
    }

    pub fn load_saved_connections(&mut self) {
        let connections_path = connections_path();
        self.connections = if connections_path.exists() {
            std::fs::read_to_string(&connections_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            vec![]
        };
    }
}

pub struct Store {
    data: Arc<RwLock<Data>>,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(Data::new())),
        }
    }

    pub fn data(&self) -> &Arc<RwLock<Data>> {
        &self.data
    }

    pub async fn run(
        &self,
        mut app_rx: UnboundedReceiver<AppCommand>,
        render_tx: UnboundedSender<RenderEvent>,
    ) {
        let data = self.data.clone();

        data.write().unwrap().load_saved_connections();
        render_tx.send(RenderEvent::Draw).ok();

        while let Some(c) = app_rx.recv().await {
            match c {
                AppCommand::SaveConnection(connection) => {
                    data.write()
                        .ok()
                        .and_then(|mut data| data.save_connection(connection).ok())
                        .and_then(|_| render_tx.send(RenderEvent::Draw).ok());
                }
                AppCommand::LoadSavedConnections => {
                    if let Ok(mut data) = data.write() {
                        data.load_saved_connections();
                        render_tx.send(RenderEvent::Draw).ok();
                    }
                }
                AppCommand::Render => {
                    render_tx.send(RenderEvent::Draw).ok();
                }
                AppCommand::Quit => {
                    break;
                }
            }
        }
    }
}

pub enum AppCommand {
    SaveConnection(ConnectionInfo),
    LoadSavedConnections,
    Render,
    Quit,
}

fn connections_path() -> std::path::PathBuf {
    dirs::data_dir()
        .expect("No data dir")
        .join("dbterm")
        .join("connections.json")
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConnectionInfo {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}
