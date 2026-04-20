use crate::tts::TtsEngine;
use std::process::Command;

pub struct WindowsTts;

impl WindowsTts {
    pub fn new() -> Self {
        Self
    }
}

impl TtsEngine for WindowsTts {
    fn speak(&self, text: &str, _lang: &str) -> Result<(), String> {
        let escaped = text.replace('\'', "\'\'");
        let script = format!(
            "Add-Type -AssemblyName System.Speech; $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; $synth.Speak('{}')",
            escaped
        );
        let output = Command::new("powershell")
            .args(["-Command", &script])
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}