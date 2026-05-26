use std::sync::Arc;

/// Voice information returned by the speech service.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language: String,
}

/// Port trait for text-to-speech functionality.
pub trait SpeechService: Send + Sync {
    /// Speak the given text. If `interrupt` is true, stop any current speech first.
    fn speak(&self, text: &str, lang: &str, interrupt: bool) -> Result<(), String>;

    /// Stop any ongoing speech.
    fn stop(&self) -> Result<(), String>;

    /// List available voices.
    fn list_voices(&self) -> Result<Vec<VoiceInfo>, String>;

    /// Set the active voice by ID.
    fn set_voice(&self, voice_id: &str) -> Result<(), String>;
}

/// Type alias for a shared speech service.
#[allow(dead_code)]
pub type SharedSpeechService = Arc<dyn SpeechService>;
