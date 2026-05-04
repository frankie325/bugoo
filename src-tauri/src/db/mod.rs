mod migrations;

use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Database not initialized")]
    NotInitialized,
    #[error("Word not found")]
    WordNotFound,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        let db = Database { conn: Mutex::new(conn) };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn run_migrations(&self) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        migrations::run(&conn)?;
        Ok(())
    }

    pub fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    pub fn connection_ref(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}
