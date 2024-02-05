use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub server: Server,
    pub database: Database,
}

#[derive(Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Deserialize)]
pub struct Database {
    pub path: PathBuf,
}
