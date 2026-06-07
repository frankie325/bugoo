# 后端开发规范

Tauri 2.x + Rust 后端，六边形架构（端口与适配器）。

---

## 规范索引

| 文件 | 描述 |
|------|------|
| [六边形架构](./hexagonal-architecture.md) | 端口/适配器/领域层的关系与边界 |
| [Commands](./commands.md) | Tauri IPC 命令编写规范 |
| [Domain Services](./domain-services.md) | 领域服务模式与依赖注入 |
| [Repository 模式](./repository-pattern.md) | Trait 定义与 SQLite 实现 |
| [Settings 系统](./settings.md) | Settings 缓存、默��值、持久化 |
| [Selection 模块](./selection.md) | 划词监听、过滤、平台适配 |
| [测试规范](./testing.md) | Rust 单元测试与集成测试 |
