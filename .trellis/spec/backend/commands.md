# Commands

## 基本形状

```rust
#[tauri::command]
pub fn get_words(state: State<'_, AppState>, search: Option<String>) -> Result<Vec<Word>, String> {
    state.word_service.get_words(search)
}
```

每个 command：
1. 加 `#[tauri::command]`
2. 通过 `State<'_, AppState>` 获取全局状态
3. 委托给 domain service
4. 返回 `Result<T, String>`（T 需实现 `Serialize`）

## 输入类型

前端 camelCase → Rust snake_case 通过 `#[serde(rename_all = "camelCase")]` 转换：

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWordInput {
    pub word: String,
    pub translation: String,
    pub source_lang: String,    // 前端 sourceLang
    pub target_lang: String,    // 前端 targetLang
    #[serde(default)]
    pub phonetic: Option<String>,
    // ...
}
```

## 注册

所有 command 必须在 `lib.rs` 的 `generate_handler![]` 中注册：

```rust
.invoke_handler(tauri::generate_handler![
    commands::words::add_word,
    commands::words::get_words,
    // ...
])
```

## 错误处理

Command 返回 `Result<T, String>`。Domain service 返回的 `Result<T, String>` 可以直接用 `?` 传播。

参考文件：
- `src-tauri/src/commands/words.rs`
- `src-tauri/src/commands/settings.rs`
- `src-tauri/src/lib.rs`（注册表）
