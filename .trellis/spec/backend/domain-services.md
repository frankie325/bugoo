# Domain Services

## 服务结构

```rust
pub struct WordService {
    repository: Arc<SqliteWordRepository>,
}

impl WordService {
    pub fn new(db: Arc<Database>) -> Self {
        WordService {
            repository: Arc::new(SqliteWordRepository::new(db)),
        }
    }
}
```

**注意**：当前实现直接用 `Arc<SqliteWordRepository>` 而非 `Arc<dyn WordRepository>`。这是务实的折中——测试时创建临时 SQLite 数据库而非 mock trait。

## 方法模式

```rust
pub fn add_word_with_details(&self, input: AddWordWithDetails) -> Result<Word, String> {
    // 1. 验证输入
    let word_text = input.word.trim().to_string();
    if word_text.is_empty() {
        return Err("单词不能为空".to_string());
    }
    // 2. 业务逻辑
    // 3. 委托 repository
    self.repository.save_with_details(&word, &detail).map_err(|e| e.to_string())
}
```

错误从 `DbError` 转换到 `String`：`.map_err(|e| e.to_string())`。

## 输入 DTO

用独立的 struct（非 domain model）接收输入：

```rust
#[derive(Debug)]
pub struct AddWordWithDetails {
    pub word: String,
    pub translation: String,
    // ...
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct WordUpdate {
    pub translation: Option<String>,
    pub tags: Option<String>,
    // ...
}
```

参考文件：
- `src-tauri/src/domain/services/word_service.rs`
- `src-tauri/src/domain/services/translation_service.rs`

## 反模式

- **不要**在 service 中直接操作数据库连接（走 repository）
- **不要**在 service 方法中 `unwrap()`（用 `?` 传播）
- **不要**把 command 层的 `serde::Deserialize` 类型直接当 service 输入（用独立的 input struct）
