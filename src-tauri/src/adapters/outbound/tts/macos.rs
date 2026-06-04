use std::process::Command;

use crate::ports::outbound::speech::VoiceInfo;

/// 构造 macOS 的 `say` 子进程命令。
pub(super) fn build_command(text: &str, lang: &str, voice_id: &Option<String>) -> Command {
    let mut command = Command::new("say");

    if let Some(voice) = voice_id {
        command.args(["-v", voice]);
    } else if is_chinese_lang(lang) {
        command.args(["-v", "Ting-Ting"]);
    }

    command.arg(text);
    command
}

/// 通过 `say -v ?` 列出系统安装的语音。
pub(super) fn list_voices() -> Result<Vec<VoiceInfo>, String> {
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

/// 停止 macOS 上正在朗读的进程。
pub(super) fn stop() -> Result<(), String> {
    Command::new("say")
        .arg("--stop")
        .spawn()
        .map_err(|e| format!("failed to stop TTS: {e}"))?;
    Ok(())
}

fn is_chinese_lang(lang: &str) -> bool {
    let normalized = lang.to_ascii_lowercase();
    normalized == "zh" || normalized.starts_with("zh-") || normalized.starts_with("zh_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_command_uses_say_with_text() {
        let command = build_command("hello", "en", &None);

        assert_eq!(command.get_program().to_string_lossy(), "say");
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["hello"]);
    }

    #[test]
    fn macos_command_uses_chinese_voice_for_chinese_lang() {
        let command = build_command("你好", "zh-CN", &None);

        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["-v", "Ting-Ting", "你好"]);
    }

    #[test]
    fn macos_command_uses_custom_voice_over_lang_hint() {
        let voice = Some("Alex".to_string());
        let command = build_command("hello", "zh-CN", &voice);

        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["-v", "Alex", "hello"]);
    }
}
