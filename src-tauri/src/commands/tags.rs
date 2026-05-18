use crate::commands::AppState;
use crate::domain::models::tag::{Tag, TagCreateInput, TagUpdateInput, TagReorderInput};
use crate::db::tags as db_tags;

/// 获取所有标签
#[tauri::command]
pub fn get_tags(state: tauri::State<AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.db.connection();
    db_tags::get_all_tags(&conn).map_err(|e| e.to_string())
}

/// 创建新标签
#[tauri::command]
pub fn create_tag(
    state: tauri::State<AppState>,
    input: TagCreateInput,
) -> Result<Tag, String> {
    let conn = state.db.connection();

    // 检查名称唯一性
    if let Some(existing) = db_tags::find_tag_by_name(&conn, &input.name).map_err(|e| e.to_string())? {
        return Err(format!("Tag '{}' already exists", existing.name));
    }

    db_tags::create_tag(&conn, input).map_err(|e| e.to_string())
}

/// 更新标签
#[tauri::command]
pub fn update_tag(
    state: tauri::State<AppState>,
    id: String,
    input: TagUpdateInput,
) -> Result<Tag, String> {
    let conn = state.db.connection();

    // 如果更新名称，检查唯一性
    if let Some(new_name) = &input.name {
        if let Some(existing) = db_tags::find_tag_by_name(&conn, new_name).map_err(|e| e.to_string())? {
            if existing.id != id {
                return Err(format!("Tag '{}' already exists", existing.name));
            }
        }
    }

    db_tags::update_tag(&conn, &id, input).map_err(|e| e.to_string())
}

/// 删除标签
#[tauri::command]
pub fn delete_tag(
    state: tauri::State<AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.connection();
    db_tags::delete_tag(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

/// 重排标签顺序
#[tauri::command]
pub fn reorder_tags(
    state: tauri::State<AppState>,
    input: TagReorderInput,
) -> Result<Vec<Tag>, String> {
    let conn = state.db.connection();
    db_tags::reorder_tags(&conn, input.tag_ids).map_err(|e| e.to_string())
}