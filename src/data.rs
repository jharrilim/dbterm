use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Data {
    pub connections: ConnectionInfo
}

impl Data {
  pub fn new() -> Self {
    Self::default()
  }
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
