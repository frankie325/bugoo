mod migrations;
pub mod tags;

use rusqlite::Connection;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        db.initialize_settings()?;
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

    pub fn initialize_settings(&self) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        let defaults = load_default_settings()?;
        for (key, value) in defaults {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
                [key.as_str(), value.as_str()],
            )?;
        }
        Ok(())
    }
}

fn load_default_settings() -> Result<HashMap<String, String>, DbError> {
    let settings_path = get_default_settings_path()?;
    if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)?;
        let map: HashMap<String, String> = serde_json::from_str(&content)?;
        Ok(map)
    } else {
        Ok(get_hardcoded_defaults())
    }
}

fn get_default_settings_path() -> Result<std::path::PathBuf, DbError> {
    let resource_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources");
    Ok(resource_dir.join("default-settings.json"))
}

fn get_hardcoded_defaults() -> HashMap<String, String> {
    HashMap::from([
        ("theme".to_string(), "light".to_string()),
        ("startup".to_string(), "false".to_string()),
        ("closeBehavior".to_string(), "minimize".to_string()),
        ("autoUpdate".to_string(), "true".to_string()),
        ("language".to_string(), "en".to_string()),
        ("dailyLimit".to_string(), "20".to_string()),
        ("reviewPace".to_string(), "normal".to_string()),
        ("hintStrategy".to_string(), "progressive".to_string()),
        ("enableSelection".to_string(), "true".to_string()),
        ("autoSpeak".to_string(), "false".to_string()),
        ("autoClose".to_string(), "true".to_string()),
        ("translationEngine".to_string(), "local".to_string()),
        ("sourceLanguage".to_string(), "auto".to_string()),
        ("targetLanguage".to_string(), "zh".to_string()),
        ("apiEndpoint".to_string(), "".to_string()),
        ("apiKey".to_string(), "".to_string()),
        ("apiSecret".to_string(), "".to_string()),
        ("apiRegion".to_string(), "".to_string()),
        ("translationModel".to_string(), "".to_string()),
        ("translationPrompt".to_string(), "".to_string()),
        ("wordDetailPrompt".to_string(), "".to_string()),
        ("translationTimeoutMs".to_string(), "15000".to_string()),
        ("themeColor".to_string(), "#10b981".to_string()),
        ("cardStyle".to_string(), "rich".to_string()),
        ("fontSize".to_string(), "medium".to_string()),
        ("reminderStartTime".to_string(), "09:00".to_string()),
        ("reminderEndTime".to_string(), "21:00".to_string()),
        ("notifyDailyReview".to_string(), "true".to_string()),
        ("notifyForgetting".to_string(), "true".to_string()),
        ("notifyStreak".to_string(), "true".to_string()),
        ("notifyAchievement".to_string(), "true".to_string()),
        ("shortcutStartReview".to_string(), "Cmd+Enter".to_string()),
        ("shortcutTranslation".to_string(), "Cmd+Shift+B".to_string()),
        ("shortcutNewWord".to_string(), "Cmd+D".to_string()),
        ("shortcutOpenApp".to_string(), "Cmd+K".to_string()),
    ])
}
