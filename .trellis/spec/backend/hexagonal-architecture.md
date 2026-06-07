# Hexagonal Architecture

## 分层结构

```
commands/        ← Tauri IPC 适配器层（接收前端调用）
    ↓
domain/services/  ← 领域服务（核心业务逻辑）
    ↓
ports/outbound/  ← 接口定义（trait）
    ↓
adapters/outbound/ ← 接口实现（SQLite、DeepL、TTS）
```

## 层间依赖规则

1. **commands/** 依赖 `AppState`（包含所有 service），委托给 domain services
2. **domain/services/** 依赖 `ports/outbound/` 的 trait，不依赖具体实现
3. **ports/outbound/** 只定义 trait，不引入任何实现依赖
4. **adapters/outbound/** 实现 trait，依赖外部 crate（rusqlite、reqwest 等）

## AppState

`AppState` 是 Tauri 的全局状态容器，持有所有 service 实例：

```rust
pub struct AppState {
    pub db: Arc<Database>,
    pub word_service: WordService,
    pub translation_service: TranslationService,
    pub settings_cache: RwLock<HashMap<String, String>>,
    // ...
}
```

## 务实简化

当前实现中某些 service 直接用 `Arc<SqliteWordRepository>` 而非 `Arc<dyn WordRepository>`。这是因为：
- 测试通过临时 SQLite 数据库而非 mock trait 完成
- 没有更换存储后端的实际需求

如果未来需要真正的 trait 解耦（如 mock 测试），改为 `Box<dyn WordRepository>` 即可。

## 模块边界

- `domain/models/` — 纯数据结构，不依赖任何外部 crate（除 serde）
- `db/` — 数据库连接、migration、schema 定义
- `scheduler/` — SM-2 间隔重复算法、通知调度
- `selection/` — 独立模块，不依赖 domain/services

## 反模式

- **不要**在 adapters 中包含业务逻辑
- **不要**在 domain models 中引用 adapters
- **不要**在 commands 中直接操作数据库
- **不要**跨层 import（commands 不能 import adapters）
