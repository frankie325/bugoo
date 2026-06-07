# Selection Module

## 模块结构

```
selection/
├── mod.rs
├── types.rs          # SelectionCandidate 等共享类型
├── filter.rs         # 文本过滤（trim、去重、长度检查）
├── gesture.rs        # 手势检测
├── debounce.rs       # 防抖逻辑
├── mouse_event.rs    # 鼠标事件处理
├── reader.rs         # 选中文本读取
├── processor.rs      # 处理流水线（整合过滤→事件→弹窗）
├── permission_prompt.rs  # 权限提示流程
├── listener/         # 全局输入监听
│   ├── mod.rs        # 跨平台 trait
│   ├── macos.rs      # macOS CGEvent 实现
│   └── rdev.rs       # rdev 跨平台实现
├── permission/       # 辅助功能权限检查
│   ├── mod.rs
│   ├── macos.rs
│   └── other.rs
└── platform/         # 平台特定事件
    ├── mod.rs
    └── macos_events.rs
```

## 数据流

```
用户选中文字
  → listener 检测 mouse up / 快捷键
  → processor 去抖 + 读取选中文本
  → filter 过滤（空文本、长度等）
  → 通过 Tauri event 发送到前端弹窗
```

## Filter 模式

`filter_selection_text(raw_text, captured_at) -> Option<SelectionCandidate>` 是纯函数，只过滤不触发副作用。

参考文件：`src-tauri/src/selection/filter.rs`

## 平台适配

用 `#[cfg(target_os = "macos")]` 条件编译实现平台差异：

```rust
// listener/mod.rs
#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
mod rdev;
```

## 测试

Filter 逻辑用单元测试，不依赖 OS 环境：

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn rejects_empty_text_after_trim() { ... }
    #[test]
    fn accepts_long_text_regardless_of_length() { ... }
}
```

参考文件：`src-tauri/src/selection/filter.rs` 底部测试

## 反模式

- **不要**在 filter 中引入副作用（网络请求、弹窗触发）
- **不要**在 listener 中包含业务逻辑（只负责检测输入事件）
- **不要**在平台无关代码中写死 macOS/CGEvent 逻辑
