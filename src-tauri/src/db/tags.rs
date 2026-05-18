use rusqlite::{params, Connection, OptionalExtension, Result};
use crate::domain::models::tag::{Tag, TagCreateInput, TagUpdateInput};

/// 初始化标签表
pub fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// 获取所有标签（按 sort_order 升序）
pub fn get_all_tags(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, sort_order, created_at, updated_at
         FROM tags ORDER BY sort_order ASC, created_at ASC",
    )?;

    let tags = stmt.query_map([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            sort_order: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;

    tags.collect()
}

/// 创建新标签
pub fn create_tag(conn: &Connection, input: TagCreateInput) -> Result<Tag> {
    let tag = Tag::new(input);

    conn.execute(
        "INSERT INTO tags (id, name, color, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![tag.id, tag.name, tag.color, tag.sort_order, tag.created_at, tag.updated_at],
    )?;

    Ok(tag)
}

/// 更新标签
pub fn update_tag(conn: &Connection, id: &str, input: TagUpdateInput) -> Result<Tag> {
    let existing = find_tag_by_id(conn, id)?
        .ok_or(rusqlite::Error::QueryReturnedNoRows)?;

    let updated = existing.apply_update(input);

    conn.execute(
        "UPDATE tags SET name = ?1, color = ?2, sort_order = ?3, updated_at = ?4 WHERE id = ?5",
        params![updated.name, updated.color, updated.sort_order, updated.updated_at, updated.id],
    )?;

    Ok(updated)
}

/// 删除标签
pub fn delete_tag(conn: &Connection, id: &str) -> Result<usize> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![id])
}

/// 重排标签顺序（批量更新 sort_order 字段）
pub fn reorder_tags(conn: &Connection, tag_ids: Vec<String>) -> Result<Vec<Tag>> {
    let now = chrono::Utc::now().timestamp_millis();

    for (index, id) in tag_ids.iter().enumerate() {
        conn.execute(
            "UPDATE tags SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
            params![index as i64, now, id],
        )?;
    }

    get_all_tags(conn)
}

/// 根据 ID 查找标签
pub fn find_tag_by_id(conn: &Connection, id: &str) -> Result<Option<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, sort_order, created_at, updated_at FROM tags WHERE id = ?1",
    )?;

    let tag = stmt
        .query_row(params![id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .optional()?;

    Ok(tag)
}

/// 根据名称查找标签（用于去重检查）
pub fn find_tag_by_name(conn: &Connection, name: &str) -> Result<Option<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, sort_order, created_at, updated_at FROM tags WHERE name = ?1",
    )?;

    let tag = stmt
        .query_row(params![name], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .optional()?;

    Ok(tag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::tag::{TagCreateInput, TagUpdateInput};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();
        conn
    }

    fn make_input(name: &str, color: &str, sort_order: i64) -> TagCreateInput {
        TagCreateInput {
            name: name.to_string(),
            color: color.to_string(),
            sort_order: Some(sort_order),
        }
    }

    #[test]
    fn create_tag_stores_all_fields() {
        let conn = setup_db();
        let input = make_input("英语", "#3b82f6", 0);
        let tag = create_tag(&conn, input).unwrap();

        assert!(!tag.id.is_empty());
        assert_eq!(tag.name, "英语");
        assert_eq!(tag.color, "#3b82f6");
        assert_eq!(tag.sort_order, 0);
        assert!(tag.created_at > 0);
        assert!(tag.updated_at > 0);
    }

    #[test]
    fn create_tag_rejects_duplicate_name() {
        let conn = setup_db();
        create_tag(&conn, make_input("英语", "#3b82f6", 0)).unwrap();
        let result = create_tag(&conn, make_input("英语", "#ef4444", 1));
        assert!(result.is_err());
    }

    #[test]
    fn get_all_tags_returns_sorted_by_sort_order() {
        let conn = setup_db();
        create_tag(&conn, make_input("C", "#ccc", 2)).unwrap();
        create_tag(&conn, make_input("A", "#aaa", 0)).unwrap();
        create_tag(&conn, make_input("B", "#bbb", 1)).unwrap();

        let tags = get_all_tags(&conn).unwrap();
        let names: Vec<&str> = tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["A", "B", "C"]);
    }

    #[test]
    fn update_tag_changes_name_and_color() {
        let conn = setup_db();
        let original = create_tag(&conn, make_input("英语", "#3b82f6", 0)).unwrap();

        let updated = update_tag(
            &conn,
            &original.id,
            TagUpdateInput {
                name: Some("EN".to_string()),
                color: Some("#ef4444".to_string()),
                sort_order: None,
            },
        ).unwrap();

        assert_eq!(updated.name, "EN");
        assert_eq!(updated.color, "#ef4444");
        assert_eq!(updated.sort_order, original.sort_order);
        assert_eq!(updated.id, original.id);
        assert!(updated.updated_at >= original.updated_at);
    }

    #[test]
    fn update_tag_rejects_duplicate_name() {
        let conn = setup_db();
        create_tag(&conn, make_input("英语", "#3b82f6", 0)).unwrap();
        let other = create_tag(&conn, make_input("法语", "#ef4444", 1)).unwrap();

        let result = update_tag(
            &conn,
            &other.id,
            TagUpdateInput {
                name: Some("英语".to_string()),
                color: None,
                sort_order: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn update_tag_allows_same_name_on_same_tag() {
        let conn = setup_db();
        let tag = create_tag(&conn, make_input("英语", "#3b82f6", 0)).unwrap();

        let result = update_tag(
            &conn,
            &tag.id,
            TagUpdateInput {
                name: Some("英语".to_string()),
                color: Some("#22c55e".to_string()),
                sort_order: None,
            },
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().color, "#22c55e");
    }

    #[test]
    fn delete_tag_removes_from_db() {
        let conn = setup_db();
        let tag = create_tag(&conn, make_input("英语", "#3b82f6", 0)).unwrap();

        let deleted = delete_tag(&conn, &tag.id).unwrap();
        assert_eq!(deleted, 1);

        let tags = get_all_tags(&conn).unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn delete_nonexistent_tag_returns_zero() {
        let conn = setup_db();
        let deleted = delete_tag(&conn, "fake_id").unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn reorder_tags_updates_sort_order() {
        let conn = setup_db();
        let a = create_tag(&conn, make_input("A", "#aaa", 0)).unwrap();
        let b = create_tag(&conn, make_input("B", "#bbb", 1)).unwrap();
        let c = create_tag(&conn, make_input("C", "#ccc", 2)).unwrap();

        let reordered = reorder_tags(&conn, vec![c.id.clone(), a.id.clone(), b.id.clone()]).unwrap();

        assert_eq!(reordered[0].name, "C");
        assert_eq!(reordered[0].sort_order, 0);
        assert_eq!(reordered[1].name, "A");
        assert_eq!(reordered[1].sort_order, 1);
        assert_eq!(reordered[2].name, "B");
        assert_eq!(reordered[2].sort_order, 2);
    }

    #[test]
    fn find_tag_by_name_returns_existing() {
        let conn = setup_db();
        create_tag(&conn, make_input("英语", "#3b82f6", 0)).unwrap();

        let found = find_tag_by_name(&conn, "英语").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().color, "#3b82f6");
    }

    #[test]
    fn find_tag_by_name_returns_none_for_missing() {
        let conn = setup_db();
        let found = find_tag_by_name(&conn, "不存在").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn update_tag_not_found_returns_error() {
        let conn = setup_db();
        let result = update_tag(
            &conn,
            "nonexistent_id",
            TagUpdateInput {
                name: Some("x".to_string()),
                color: None,
                sort_order: None,
            },
        );
        assert!(result.is_err());
    }
}