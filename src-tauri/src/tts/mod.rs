use std::process::Command;

pub fn speak_text(text: &str, lang: Option<&str>) -> Result<(), String> {
    let Some(text) = normalize_text(text) else {
        return Ok(());
    };

    let mut command = build_system_tts_command(&text, lang);
    command
        .spawn()
        .map_err(|error| format!("failed to start system TTS: {error}"))?;

    Ok(())
}

fn normalize_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn build_system_tts_command(text: &str, lang: Option<&str>) -> Command {
    platform_tts_command(text, lang)
}

#[cfg(target_os = "macos")]
fn platform_tts_command(text: &str, lang: Option<&str>) -> Command {
    let mut command = Command::new("say");

    if is_chinese_lang(lang) {
        command.args(["-v", "Ting-Ting"]);
    }

    command.arg(text);
    command
}

#[cfg(target_os = "windows")]
fn platform_tts_command(text: &str, _lang: Option<&str>) -> Command {
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
fn platform_tts_command(text: &str, _lang: Option<&str>) -> Command {
    let mut command = Command::new("spd-say");
    command.arg(text);
    command
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_tts_command(_text: &str, _lang: Option<&str>) -> Command {
    Command::new("__bugoo_unsupported_tts_platform__")
}

#[cfg(target_os = "macos")]
fn is_chinese_lang(lang: Option<&str>) -> bool {
    lang.map(|value| {
        let normalized = value.to_ascii_lowercase();
        normalized == "zh" || normalized.starts_with("zh-") || normalized.starts_with("zh_")
    })
    .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_text_returns_none_for_blank_input() {
        assert_eq!(normalize_text(""), None);
        assert_eq!(normalize_text("   \n\t  "), None);
    }

    #[test]
    fn normalize_text_trims_non_blank_input() {
        assert_eq!(normalize_text("  hello  "), Some("hello".to_owned()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_command_uses_say_with_text() {
        let command = build_system_tts_command("hello", Some("en"));

        assert_eq!(command.get_program().to_string_lossy(), "say");
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["hello"]);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_command_uses_chinese_voice_for_chinese_lang_hint() {
        let command = build_system_tts_command("你好", Some("zh-CN"));

        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["-v", "Ting-Ting", "你好"]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_command_uses_powershell_system_speech() {
        let command = build_system_tts_command("Bob's word", Some("en"));

        assert_eq!(command.get_program().to_string_lossy(), "powershell");
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args[0], "-NoProfile");
        assert_eq!(args[1], "-Command");
        assert!(args[2].contains("System.Speech.Synthesis.SpeechSynthesizer"));
        assert!(args[2].contains("'Bob''s word'"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_command_uses_spd_say_with_text() {
        let command = build_system_tts_command("hello", Some("en"));

        assert_eq!(command.get_program().to_string_lossy(), "spd-say");
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["hello"]);
    }
}
