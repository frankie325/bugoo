use crate::adapters::outbound::tts::SystemSpeechAdapter;
use crate::ports::outbound::speech::SpeechService;

pub struct SpeechServiceInstance {
    adapter: SystemSpeechAdapter,
}

impl SpeechServiceInstance {
    pub fn new() -> Result<Self, String> {
        Ok(SpeechServiceInstance {
            adapter: SystemSpeechAdapter::new()?,
        })
    }

    pub fn speak(&self, text: &str, lang: &str, interrupt: bool) -> Result<(), String> {
        self.adapter.speak(text, lang, interrupt)
    }

    pub fn stop(&self) -> Result<(), String> {
        self.adapter.stop()
    }

    pub fn list_voices(&self) -> Result<Vec<crate::ports::outbound::speech::VoiceInfo>, String> {
        self.adapter.list_voices()
    }

    pub fn set_voice(&self, voice_id: &str) -> Result<(), String> {
        self.adapter.set_voice(voice_id)
    }
}
