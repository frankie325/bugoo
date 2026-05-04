#[tauri::command]
pub fn speak_text(_text: String, _lang: String) -> Result<(), String> {
    Ok(())
}
