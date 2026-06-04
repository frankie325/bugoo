use std::sync::Mutex;

use crate::ports::outbound::speech::{SpeechService, VoiceInfo};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod unsupported;

#[cfg(target_os = "macos")]
use macos::{build_command, list_voices, stop};
#[cfg(target_os = "windows")]
use windows::{build_command, list_voices, stop};
#[cfg(target_os = "linux")]
use linux::{build_command, list_voices, stop};
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
use unsupported::{build_command, list_voices, stop};

/// Adapter that uses system commands for speech synthesis.
pub struct SystemSpeechAdapter {
    engine: Mutex<SystemTtsEngine>,
}

struct SystemTtsEngine {
    voice_id: Option<String>,
}

impl SystemSpeechAdapter {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            engine: Mutex::new(SystemTtsEngine { voice_id: None }),
        })
    }
}

impl SpeechService for SystemSpeechAdapter {
    fn speak(&self, text: &str, lang: &str, interrupt: bool) -> Result<(), String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        if interrupt {
            self.stop()?;
        }

        let engine = self
            .engine
            .lock()
            .map_err(|e| format!("TTS lock failed: {e}"))?;
        let mut command = build_command(trimmed, lang, &engine.voice_id);
        command
            .spawn()
            .map_err(|e| format!("failed to start system TTS: {e}"))?;

        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        stop()
    }

    fn list_voices(&self) -> Result<Vec<VoiceInfo>, String> {
        list_voices()
    }

    fn set_voice(&self, voice_id: &str) -> Result<(), String> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|e| format!("TTS lock failed: {e}"))?;
        engine.voice_id = Some(voice_id.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_speak_skips_empty_text() {
        let adapter = SystemSpeechAdapter::new().unwrap();
        let result = adapter.speak("", "en", false);
        assert!(result.is_ok());
    }

    #[test]
    fn adapter_set_voice_stores_voice_id() {
        let adapter = SystemSpeechAdapter::new().unwrap();
        adapter.set_voice("Alex").unwrap();
        let engine = adapter.engine.lock().unwrap();
        assert_eq!(engine.voice_id, Some("Alex".to_string()));
    }
}
