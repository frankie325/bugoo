use std::process::Command;

use crate::ports::outbound::speech::VoiceInfo;

/// 构造 Linux 的 `spd-say`（speech-dispatcher）子进程命令。
pub(super) fn build_command(text: &str, _lang: &str, _voice_id: &Option<String>) -> Command {
    let mut command = Command::new("spd-say");
    command.arg(text);
    command
}

/// speech-dispatcher 不提供统一 CLI 的语音查询。
pub(super) fn list_voices() -> Result<Vec<VoiceInfo>, String> {
    Ok(Vec::new())
}

/// 通过 `spd-say --stop` 停止当前朗读。
pub(super) fn stop() -> Result<(), String> {
    Command::new("spd-say")
        .arg("--stop")
        .spawn()
        .map_err(|e| format!("failed to stop TTS: {e}"))?;
    Ok(())
}
