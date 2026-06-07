# Settings System

## 架构

Settings 以 `key-value` 对形式存储在 SQLite `settings` 表中，运行时通过 `RwLock<HashMap<String, String>>` 缓存。

## 存储层

```rust
// AppState 中
pub settings_cache: RwLock<HashMap<String, String>>,
```

读写缓存通过 helper 方法：

```rust
impl AppState {
    pub fn settings_cache_read(&self) -> HashMap<String, String> {
        self.settings_cache.read().unwrap().clone()
    }

    pub fn settings_cache_reload(&self) { ... }
}
```

参考文件：`src-tauri/src/commands/mod.rs`

## 默认值注入

两层回退：
1. `src-tauri/resources/default-settings.json` — 打包的资源文件
2. `src-tauri/src/db/mod.rs` — 硬编码回退

`seed_settings` command 在应用首次启动时将 JSON 默认值写入 settings 表。

## 前端使用

```ts
// 读取
const settings = useSettingsStore((state) => state.settings);
const enableSelection = settings.enableSelection !== "false";

// 写入（乐观更新 + 持久化）
updateSetting("enableSelection", String(val));
```

Store 中 `updateSetting` 先更新 Zustand 状态，再异步调 `setSetting` IPC 持久化。持久化失败不阻塞 UI。

参考文件：
- `src-tauri/src/commands/settings.rs`
- `src/stores/settingsStore.ts`
- `src/lib/api/settings.ts`

## 典型 key

| Key | 类型 | 默认值 |
|-----|------|--------|
| `enableSelection` | bool string | `"true"` |
| `autoSpeak` | bool string | `"false"` |
| `autoClose` | bool string | `"true"` |
| `dailyLimit` | number string | `"20"` |
| `reviewPace` | enum string | `"normal"` |
| `selectionLimitEnabled` | bool string | `"true"` |
| `maxSelectionChars` | number string | `"200"` |

## 反模式

- **不要**在每次读 settings 时查 SQLite（用缓存）
- **不要**在循环中反复读锁（先 clone 再使用）
- **不要**新增 settings key 但不在 `default-settings.json` 加默认值
