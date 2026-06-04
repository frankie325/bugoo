use std::process::Command;

use crate::ports::outbound::speech::VoiceInfo;

/// 构造调用 .NET `System.Speech` 的 PowerShell 命令。
pub(super) fn build_command(text: &str, _lang: &str, _voice_id: &Option<String>) -> Command {
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

/// Windows 平台暂未实现语音列表查询。
pub(super) fn list_voices() -> Result<Vec<VoiceInfo>, String> {
    Ok(Vec::new())
}

/// Windows 的 TTS 进程由 PowerShell `Speak()` 同步执行，没有可停止的子进程。
pub(super) fn stop() -> Result<(), String> {
    Ok(())
}

fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
