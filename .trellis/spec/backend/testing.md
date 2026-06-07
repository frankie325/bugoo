# Testing

## 单元测试

放在被测文件底部 `#[cfg(test)] mod tests { ... }`：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_by_text_matches_language_case_insensitively() {
        let db = test_db();
        let repo = SqliteWordRepository::new(Arc::clone(&db));
        // Arrange
        let word = Word::new("word-1".into(), "Hello".into(), "你好".into(), "EN".into(), "ZH".into());
        repo.save_with_details(&word, &detail_draft()).unwrap();

        // Act
        let found = repo.find_by_text("hello", "zh").unwrap().expect("word should be found");

        // Assert
        assert_eq!(found.id, "word-1");
    }
}
```

## 测试数据库

测试用临时 SQLite 数据库，避免影响真实数据：

```rust
fn test_db() -> Arc<Database> {
    let path = std::env::temp_dir().join(format!("bugoo-sqlite-repo-{}.db", uuid::Uuid::new_v4()));
    Arc::new(Database::new(path).unwrap())
}
```

## 测试命名

描述性名称，用 `_` 分隔：
- `rejects_empty_text_after_trim`
- `find_by_text_matches_language_case_insensitively`
- `save_with_details_upserts_detail_for_existing_word_without_duplicate`

## 运行命令

```bash
cd src-tauri && cargo test          # 全部测试
cd src-tauri && cargo test --lib    # 仅单元测试
cd src-tauri && cargo test filter   # 按名称过滤
```

## 测试文件分布

| 位置 | 测试文件 |
|------|---------|
| `src-tauri/src/domain/models/word.rs` | `is_valid_word_form_type` 测试 |
| `src-tauri/src/domain/services/word_service.rs` | `add_word_with_details_reuses_existing_word` 等 |
| `src-tauri/src/adapters/outbound/sqlite.rs` | SQLite CRUD 测试（最多、最详细） |
| `src-tauri/src/selection/filter.rs` | 文本过滤测试 |
| `src-tauri/src/selection/listener/mod.rs` | 弹窗评估测试 |

## 反模式

- **不要**写测试但没有断言（至少一个 `assert!`）
- **不要**测试依赖真实外部 API（DeepL、TTS 等）
- **不要**在测试中使用生产数据库路径
