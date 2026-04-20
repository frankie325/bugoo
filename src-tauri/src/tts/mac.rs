use crate::tts::TtsEngine;
use std::process::Command;

pub struct MacTts;

impl MacTts {
    pub fn new() -> Self {
        Self
    }
}

impl TtsEngine for MacTts {
    fn speak(&self, text: &str, _lang: &str) -> Result<(), String> {
        let output = Command::new("say")
            .arg(text)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}