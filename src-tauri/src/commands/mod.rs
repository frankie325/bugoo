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
use std::sync::Arc;

pub struct AppState {
    pub db: Arc<Database>,
    pub word_service: WordService,
    pub speech_service: SpeechServiceInstance,
    pub translation_service: TranslationService,
}

impl AppState {
    pub fn new(db: Arc<Database>, translation_service: TranslationService) -> Self {
        AppState {
            db: db.clone(),
            word_service: WordService::new(db),
            speech_service: SpeechServiceInstance::new()
                .expect("Failed to initialize speech service"),
            translation_service,
        }
    }
}
