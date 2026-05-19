use crate::commands::AppState;
use tauri::State;

#[tauri::command]
pub fn speak_text(
    text: String,
    lang: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let lang_str = lang.as_deref().unwrap_or("en");
    state.speech_service.speak(&text, lang_str, false)
}

#[tauri::command]
pub fn stop_speech(state: State<'_, AppState>) -> Result<(), String> {
    state.speech_service.stop()
}

#[tauri::command]
pub fn list_voices(state: State<'_, AppState>) -> Result<Vec<crate::ports::outbound::speech::VoiceInfo>, String> {
    state.speech_service.list_voices()
}

#[tauri::command]
pub fn set_voice(voice_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.speech_service.set_voice(&voice_id)
}