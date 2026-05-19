pub mod review;
pub mod settings;
pub mod tags;
pub mod translate;
pub mod tts;
pub mod window;
pub mod word_details;
pub mod words;

use crate::db::Database;
use crate::domain::services::word_service::WordService;
use std::sync::Arc;

pub struct AppState {
    pub db: Arc<Database>,
    pub word_service: WordService,
}

impl AppState {
    pub fn new(db: Arc<Database>) -> Self {
        AppState {
            db: db.clone(),
            word_service: WordService::new(db),
        }
    }
}
