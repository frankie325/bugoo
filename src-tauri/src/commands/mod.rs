pub mod review;
pub mod settings;
pub mod tags;
pub mod translate;
pub mod translation_languages;
pub mod tts;
pub mod window;
pub mod word_details;
pub mod words;
use crate::db::Database;
use crate::domain::services::speech_service::SpeechServiceInstance;
use crate::domain::services::translation_service::TranslationService;
use crate::domain::services::word_service::WordService;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct AppState {
    pub db: Arc<Database>,
    pub word_service: WordService,
    pub speech_service: SpeechServiceInstance,
    pub translation_service: TranslationService,
    pub(crate) settings_cache: RwLock<HashMap<String, String>>,
}

impl AppState {
    pub fn new(db: Arc<Database>, translation_service: TranslationService) -> Self {
        let settings_cache =
            read_settings_map_from_db(db.connection()).unwrap_or_else(|_| HashMap::new());
        AppState {
            db: db.clone(),
            word_service: WordService::new(db),
            speech_service: SpeechServiceInstance::new()
                .expect("Failed to initialize speech service"),
            translation_service,
            settings_cache: RwLock::new(settings_cache),
        }
    }

    pub fn settings_cache_read(&self) -> Result<HashMap<String, String>, String> {
        let cache = self.settings_cache.read().map_err(|e| e.to_string())?;
        Ok(cache.clone())
    }

    pub fn settings_cache_reload(&self) -> Result<(), String> {
        let conn = self.db.connection();
        let settings = read_settings_map_from_db(conn)?;
        let mut cache = self.settings_cache.write().map_err(|e| e.to_string())?;
        *cache = settings;
        Ok(())
    }
}

fn read_settings_map_from_db(
    conn: std::sync::MutexGuard<'_, rusqlite::Connection>,
) -> Result<HashMap<String, String>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?;

    let mut settings = HashMap::new();
    for row in rows {
        let (key, value) = row.map_err(|error| error.to_string())?;
        settings.insert(key, value);
    }
    Ok(settings)
}
