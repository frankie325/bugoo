use crate::ports::outbound::speech::{SpeechService, VoiceInfo};
use std::process::Command;
use std::sync::Mutex;

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
        let mut command = build_system_tts_command(trimmed, lang, &engine.voice_id);
        command
            .spawn()
            .map_err(|e| format!("failed to start system TTS: {e}"))?;

        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            Command::new("say")
                .arg("--stop")
                .spawn()
                .map_err(|e| format!("failed to stop TTS: {e}"))?;
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("spd-say")
                .arg("--stop")
                .spawn()
                .map_err(|e| format!("failed to stop TTS: {e}"))?;
        }
        Ok(())
    }

    fn list_voices(&self) -> Result<Vec<VoiceInfo>, String> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("say")
                .arg("-v")
                .arg("?")
                .output()
                .map_err(|e| format!("failed to list voices: {e}"))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let voices: Vec<VoiceInfo> = stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
                    if parts.len() >= 2 {
                        let lang_part = parts[1].trim();
                        Some(VoiceInfo {
                            id: parts[0].to_string(),
                            name: parts[0].to_string(),
                            language: lang_part.to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            Ok(voices)
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(Vec::new())
        }
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

fn build_system_tts_command(text: &str, lang: &str, voice_id: &Option<String>) -> Command {
    platform_tts_command(text, lang, voice_id)
}

#[cfg(target_os = "macos")]
fn platform_tts_command(text: &str, lang: &str, voice_id: &Option<String>) -> Command {
    let mut command = Command::new("say");

    if let Some(voice) = voice_id {
        command.args(["-v", voice]);
    } else if is_chinese_lang(lang) {
        command.args(["-v", "Ting-Ting"]);
    }

    command.arg(text);
    command
}

#[cfg(target_os = "windows")]
fn platform_tts_command(text: &str, _lang: &str, _voice_id: &Option<String>) -> Command {
    let mut command = Command::new("powershell");
    command.args([
        "-NoProfile",
        "-Command",
        &format!(
            "Add-Type -AssemblyName System.Speech; \
             $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
             $s.Speak({});",
            powershell_single_quoted(text)
        ),
    ]);
    command
}

#[cfg(target_os = "linux")]
fn platform_tts_command(text: &str, _lang: &str, _voice_id: &Option<String>) -> Command {
    let mut command = Command::new("spd-say");
    command.arg(text);
    command
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_tts_command(_text: &str, _lang: &str, _voice_id: &Option<String>) -> Command {
    Command::new("__bugoo_unsupported_tts_platform__")
}

#[cfg(target_os = "macos")]
fn is_chinese_lang(lang: &str) -> bool {
    let normalized = lang.to_ascii_lowercase();
    normalized == "zh" || normalized.starts_with("zh-") || normalized.starts_with("zh_")
}

#[cfg(target_os = "windows")]
fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_command_uses_say_with_text() {
        let command = build_system_tts_command("hello", "en", &None);

        assert_eq!(command.get_program().to_string_lossy(), "say");
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["hello"]);
    }

    #[test]
    fn macos_command_uses_chinese_voice_for_chinese_lang() {
        let command = build_system_tts_command("你好", "zh-CN", &None);

        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["-v", "Ting-Ting", "你好"]);
    }

    #[test]
    fn macos_command_uses_custom_voice_over_lang_hint() {
        let voice = Some("Alex".to_string());
        let command = build_system_tts_command("hello", "zh-CN", &voice);

        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["-v", "Alex", "hello"]);
    }

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
