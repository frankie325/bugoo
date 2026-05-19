#[tauri::command]
pub fn speak_text(text: String, lang: Option<String>) -> Result<(), String> {
    crate::tts::speak_text(&text, lang.as_deref())
}
