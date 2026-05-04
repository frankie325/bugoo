#[tauri::command]
pub fn translate_text(_text: String, _source_lang: String, _target_lang: String) -> Result<String, String> {
    Ok("Translation not implemented yet".to_string())
}
