//! 兜底模块：覆盖了 macos / windows / linux 之外的平台。
//! 调用时直接返回占位命令，由 `speak` 时的 `spawn` 报错暴露给用户。

use std::process::Command;

use crate::ports::outbound::speech::VoiceInfo;

pub(super) fn build_command(_text: &str, _lang: &str, _voice_id: &Option<String>) -> Command {
    Command::new("__bugoo_unsupported_tts_platform__")
}

pub(super) fn list_voices() -> Result<Vec<VoiceInfo>, String> {
    Ok(Vec::new())
}

pub(super) fn stop() -> Result<(), String> {
    Ok(())
}
