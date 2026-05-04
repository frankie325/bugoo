use crate::domain::models::Word;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_due_reviews(state: State<'_, AppState>) -> Result<Vec<Word>, String> {
    let _ = state;
    Ok(vec![])
}

#[tauri::command]
pub fn submit_review(
    state: State<'_, AppState>,
    _word_id: String,
    _rating: u8,
) -> Result<Word, String> {
    let _ = state;
    Err("Review not implemented yet".to_string())
}
