use std::fs;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::{config::Config, database::Db};

pub struct App {
    pub config: Config,
    pub db: Db,
}

impl App {
    pub fn new() -> Result<Self> {
        let raw_config = fs::read_to_string("config.toml").context("Failed to read config.toml")?;
        let config =
            toml::from_str::<Config>(&raw_config).context("Failed to parse config.toml")?;

        let connection = Connection::open(&config.database.path)
            .context("Failed to open database connection")?;
        let db = Db::new(connection);
        db.init().context("Failed to initialize database")?;

        Ok(Self { config, db })
    }

    pub fn cleanup(&self) -> Result<()> {
        if !self.db.is_active() {
            println!("[!] Already shuting down");
            return Ok(());
        }

        println!("[*] Shutting down...");
        self.db.cleanup().context("Failed to cleanup database")
    }
}
