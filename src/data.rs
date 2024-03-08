use std::{any::Any, sync::{Arc, RwLock}};

use color_eyre::eyre::Result;
use dbterm_widgets::status_line::Status;
use serde::{Deserialize, Serialize};
use sqlx::{Column, PgPool, Row, TypeInfo};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::render::{self, RenderEvent};

pub type Ctx = RwLock<Data>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Data {
    pub connections: Vec<(usize, ConnectionInfo)>,
}

impl Data {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save_new_connection(&mut self, connection: NewConnectionInfo) -> Result<()> {
        let connections_path = connections_path();
        if !connections_path.exists() {
            std::fs::create_dir_all(connections_path.parent().unwrap())?;
        }

        let id = self.connections.len();
        self.connections
            .push((id, connection.to_connection_info(id)));
        let content = serde_json::to_string_pretty(&self.connections)?;
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

    pub fn delete_connection(&mut self, id: usize) {
        self.connections.retain(|(i, _)| *i != id);
        let content = serde_json::to_string_pretty(&self.connections).unwrap();
        std::fs::write(connections_path(), content).unwrap();
    }

    pub fn connections(&self) -> Vec<ConnectionInfo> {
        // Cloning lets us avoid holding the read lock for as long as any returned reference
        self.connections.iter().map(|(_, c)| c.clone()).collect()
    }
}

pub struct Store {
    data: Arc<RwLock<Data>>,
    pool: Option<PgPool>,
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
            pool: None,
        }
    }

    pub fn data(&self) -> &Arc<RwLock<Data>> {
        &self.data
    }

    pub async fn run(
        &mut self,
        mut app_rx: UnboundedReceiver<AppCommand>,
        render_tx: UnboundedSender<RenderEvent>,
    ) -> Result<()> {
        let data = self.data.clone();

        data.write().unwrap().load_saved_connections();
        render_tx.send(RenderEvent::Draw).ok();

        while let Some(command) = app_rx.recv().await {
            match self.handle_command(command, &render_tx).await {
                Ok(true) => break,
                Ok(false) => {}
                Err(e) => {
                    render_tx
                        .send(RenderEvent::StatusMessage(Status::Error(e.to_string())))
                        .ok();
                }
            }
        }
        Ok(())
    }

    async fn handle_command(
        &mut self,
        command: AppCommand,
        render_tx: &UnboundedSender<RenderEvent>,
    ) -> Result<bool> {
        match command {
            AppCommand::SaveConnection(connection) => {
                self.data
                    .write()
                    .ok()
                    .and_then(|mut data| data.save_new_connection(connection).ok())
                    .and_then(|_| render_tx.send(RenderEvent::Draw).ok());
            }
            AppCommand::LoadSavedConnections => {
                if let Ok(mut data) = self.data.write() {
                    data.load_saved_connections();
                    render_tx.send(RenderEvent::Draw).ok();
                }
            }
            AppCommand::ConnectToDatabase(idx) => {
                if let Some((_, connection)) = self.data.read().unwrap().connections.get(idx) {
                    match connection.database_type {
                        DatabaseType::Postgres => {
                            let pool = PgPool::connect(&connection.to_connection_string()).await?;
                            self.pool = Some(pool);
                            render_tx.send(RenderEvent::Connected).ok();
                        }
                        _ => todo!("Connect to other databases"),
                    }
                }
            }
            AppCommand::DeleteConnection(idx) => {
                render_tx.send(RenderEvent::Draw).ok();
            }
            AppCommand::Query(query) => {
                if let Some(pool) = &self.pool {
                    let rows = sqlx::query(&query).fetch_all(pool).await?;

                    let headers = rows
                        .first()
                        .map(|row| row
                            .columns()
                            .iter()
                            .map(|col| col.name().to_string())
                            .collect::<Vec<String>>()
                        )
                        .unwrap_or_default();
                    let rows = rows
                        .iter()
                        .map(|row| {
                            row.columns()
                                .iter()
                                .enumerate()
                                .map(|(i, col)| {
                                    col.type_info().name().to_string()
                                })
                                .collect()
                        })
                        .collect();
                    render_tx
                        .send(RenderEvent::QueryResult { headers, rows })
                        .ok();
                }
            }
            AppCommand::Render => {
                render_tx.send(RenderEvent::Draw).ok();
            }
            AppCommand::Quit => {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

pub enum AppCommand {
    SaveConnection(NewConnectionInfo),
    LoadSavedConnections,
    ConnectToDatabase(usize),
    DeleteConnection(usize),
    Query(String),
    Render,
    Quit,
}

fn connections_path() -> std::path::PathBuf {
    dirs::data_dir()
        .expect("No data dir")
        .join("dbterm")
        .join("connections.json")
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum DatabaseType {
    Postgres,
    Mysql,
    Sqlite,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConnectionInfo {
    /// An internal id for the connection
    pub id: usize,
    /// Name of the connection in the UI, not a part of the actual connection
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub database_type: DatabaseType,
}

impl ConnectionInfo {
    pub fn to_connection_string(&self) -> String {
        match self.database_type {
            DatabaseType::Postgres => format!(
                "postgres://{}:{}@{}:{}/{}",
                self.user, self.password, self.host, self.port, self.database
            ),
            DatabaseType::Mysql => format!(
                "mysql://{}:{}@{}:{}/{}",
                self.user, self.password, self.host, self.port, self.database
            ),
            DatabaseType::Sqlite => format!("sqlite://{}", self.database),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewConnectionInfo {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub database_type: DatabaseType,
}

impl NewConnectionInfo {
    pub fn to_connection_info(self, id: usize) -> ConnectionInfo {
        ConnectionInfo {
            id,
            name: self.name,
            host: self.host,
            port: self.port,
            user: self.user,
            password: self.password,
            database: self.database,
            database_type: self.database_type,
        }
    }
}
