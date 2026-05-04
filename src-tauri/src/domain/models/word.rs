use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub id: String,
    pub word: String,
    pub translation: String,
    #[serde(default)]
    pub phonetic: Option<String>,
    #[serde(default = "default_source_lang")]
    pub source_lang: String,
    #[serde(default = "default_target_lang")]
    pub target_lang: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub audio_url: Option<String>,
    #[serde(default = "default_ease_factor")]
    pub ease_factor: f64,
    #[serde(default)]
    pub interval: i32,
    #[serde(default)]
    pub repetitions: i32,
    #[serde(default)]
    pub next_review_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn default_source_lang() -> String {
    "EN".to_string()
}

fn default_target_lang() -> String {
    "ZH".to_string()
}

fn default_status() -> String {
    "new".to_string()
}

fn default_ease_factor() -> f64 {
    2.5
}

impl Word {
    pub fn new(
        id: String,
        word: String,
        translation: String,
        source_lang: String,
        target_lang: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Word {
            id,
            word,
            translation,
            phonetic: None,
            source_lang,
            target_lang,
            status: "new".to_string(),
            tags: String::new(),
            notes: String::new(),
            audio_url: None,
            ease_factor: 2.5,
            interval: 0,
            repetitions: 0,
            next_review_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}
