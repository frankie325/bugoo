use rusqlite::Connection;
use std::path::PathBuf;
use tokio::sync::Mutex;

pub mod migrations;
pub mod models;

pub use models::Word;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(&path)?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        migrations::run_migrations(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }
}
