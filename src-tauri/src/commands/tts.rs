#[tauri::command]
pub fn speak_text(text: String, _lang: Option<String>) -> Result<(), String> {
    let tts = crate::tts::new_tts();
    tts.speak(&text, _lang.as_deref().unwrap_or(""))
}
