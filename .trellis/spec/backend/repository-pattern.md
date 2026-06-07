# Repository Pattern

## Trait 定义

```rust
pub trait WordRepository: Send + Sync {
    fn find_all(&self, search: Option<&str>) -> Result<Vec<Word>, DbError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Word>, DbError>;
    fn find_by_text(&self, word: &str, target_lang: &str) -> Result<Option<Word>, DbError>;
    fn save_with_details(&self, word: &Word, detail: &WordDetailDraft) -> Result<Word, DbError>;
    fn update(&self, word: &Word) -> Result<Word, DbError>;
    fn delete(&self, id: &str) -> Result<(), DbError>;
}
```

Trait 定义在 `ports/outbound/repository.rs`，必须 `Send + Sync`。

## SQLite 实现

```rust
pub struct SqliteWordRepository {
    db: Arc<Database>,
}

impl SqliteWordRepository {
    pub fn new(db: Arc<Database>) -> Self { ... }

    fn row_to_word(row: &rusqlite::Row) -> Result<Word, rusqlite::Error> { ... }
}

impl WordRepository for SqliteWordRepository {
    fn find_all(&self, search: Option<&str>) -> Result<Vec<Word>, DbError> {
        let conn = self.db.connection();
        // 用参数化查询，防止 SQL 注入
        let mut stmt = conn.prepare("SELECT * FROM words WHERE ...").map_err(DbError::Sqlite)?;
        // ...
    }
}
```

**关键规则**：
- 永远用参数化查询 `params![]`，禁止字符串拼接 SQL
- 用 `self.db.connection()` 获取连接
- 错误用 `DbError::Sqlite` 包装

## 错误类型

```rust
// db/mod.rs
pub enum DbError {
    Sqlite(rusqlite::Error),
    Serde(serde_json::Error),
    // ...
}
```

参考文件：
- `src-tauri/src/ports/outbound/repository.rs`
- `src-tauri/src/adapters/outbound/sqlite.rs`
- `src-tauri/src/db/mod.rs`

## 反模式

- **禁止** SQL 字符串拼接/format
- **禁止** `unwrap()` 在 repository 实现中（用 `?`）
- **禁止** 在 repository 中包含业务逻辑
