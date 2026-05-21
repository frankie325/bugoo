# 选中文字弹窗最小实现设计

日期：2026-05-21

## 背景

`docs/selection-popup.md` 定义了完整的划词翻译弹窗能力，包括翻译、复制、加入生词本、详情入口、定位和动效。本次只实现第一步：用户在系统任意应用中选中文字后打开一个小弹窗，弹窗内容只显示被选中的文字。

本设计不实现翻译请求、生词保存、详情入口、复制按钮、关闭按钮、发音和快速复习。

## 目标

- 在桌面环境中监听用户选中文字后的鼠标左键释放行为。
- 使用 `get_selected_text` 工具包获取当前系统选区中的文本。
- 当文本非空且长度不超过 50 个字符时，打开或更新一个独立的 selection popup 浮窗。
- 浮窗只展示选中的原文。
- macOS 每次启动时检查 Accessibility 权限；未授权时提醒用户跳转系统设置开启权限。
- 权限不足、读取失败或读取结果为空时不崩溃、不弹空窗，并记录可理解的失败原因。

## 非目标

- 不实现翻译结果展示。
- 不实现保存到生词本。
- 不实现词典、发音、复习、详情页跳转。
- 不实现完整的选区边界定位算法。
- 不处理图片、富文本、文件等非纯文本选区内容。
- 不在 Bugoo 内自行维护 macOS Accessibility 取词、Windows UI Automation 取词或剪贴板兜底逻辑；跨平台取词交给 `get_selected_text` 工具包。

## 推荐方案

采用“`rdev` 全局鼠标监听 + `get_selected_text` 读取选区文本 + macOS Accessibility 启动检查”的方案。

流程如下：

1. 后端在应用启动时执行平台初始化。
2. 如果当前系统是 macOS，先检查 Accessibility 权限。
3. macOS 未授权时，打开一个应用内权限提示窗口，提供“去系统设置”和“稍后”操作；本次运行只提示一次，并且不启动划词监听。
4. macOS 已授权或非 macOS 平台时，使用 `rdev` 注册全局鼠标事件监听。
5. 监听到鼠标左键释放后，调用 `get_selected_text` 工具包读取当前系统选中文本。
6. 如果读取结果 trim 后为空，则直接结束，不打开也不更新弹窗。
7. 对非空候选文本做过滤：长度超过 50 个字符不弹，500ms 内重复触发不弹。
8. 调用窗口命令打开或更新 selection popup 浮窗。

该方案把系统监听差异交给 `rdev`，把跨平台取词差异交给 `get_selected_text`。Bugoo 自身只负责权限入口、事件编排、文本过滤和弹窗管理。

## 备选方案

### 快捷键触发

用户选中文字后按快捷键，应用再读取选区并打开弹窗。该方案权限和误触更少，也能复用项目已有 global shortcut 基础，但不满足首版希望的“选中文字即弹窗”体验。

### 自研平台取词

Bugoo 自己维护 macOS Accessibility、Windows UI Automation 和剪贴板兜底链路。该方案可控性更高，但当前会重复 `get_selected_text` 工具包已经承担的跨平台兼容工作，也会增加权限、剪贴板恢复和平台边界测试成本。因此本轮不采用。

## 后端设计

### Selection 编排服务

新增或调整一个独立 selection 模块，用于处理启动权限检查、全局鼠标释放监听、选区文本读取、过滤和弹窗触发。该模块属于应用编排层，不承载翻译或生词业务逻辑。

模块职责：

- 应用启动时执行 selection 初始化。
- macOS 上检查 Accessibility 权限。
- 缺少 Accessibility 权限时，打开权限提示窗口；本次运行只提示一次。
- 权限满足时，使用 `rdev` 启动全局鼠标监听。
- 在鼠标左键释放后调用 `get_selected_text`。
- 读取结果为空时静默忽略，不打开弹窗。
- 读取结果非空时交给过滤器，再打开或更新 selection popup。

### 权限提示窗口

新增一个 macOS Accessibility 权限提示窗口，作为独立 Tauri 页面或复用窗口命令创建。

窗口行为：

- 固定 label，例如 `accessibility-permission`.
- 在 macOS 启动检查发现未授权时打开。
- 本次运行只打开一次；用户点击“稍后”后关闭窗口，不再重复弹出。
- 提供“去系统设置”按钮，调用后端命令跳转到 macOS Accessibility 设置页。
- 如果用户稍后在系统设置中授权，需要重启应用或重新触发权限检查后再启动划词监听；首版不要求实时监听权限变化。

### Selection Popup 窗口命令

在 `commands/window.rs` 中保留 selection popup 专用命令 `open_selection_popup`，保留现有 `open_float_window` 行为不变。

窗口行为：

- 使用固定 label，例如 `selection-popup`。
- 如果窗口不存在，则创建窗口并加载 `/selection-popup?text=...`。
- 如果窗口已存在，则通过 Tauri 事件发送最新文本，由前端页面更新展示内容。
- 窗口置顶，小尺寸，无装饰。
- 默认显示在鼠标释放位置附近；若无法获取坐标，则使用主屏幕安全位置。
- 后端不会用空文本打开或更新该窗口。

### 过滤与节流

过滤规则放在可单测的纯逻辑中：

- `trim()` 后为空：忽略。
- 字符数大于 50：忽略。
- 距离上次成功触发小于 500ms：忽略。
- 短时间内文本完全相同：忽略。

## 前端设计

新增 `src/pages/SelectionPopup/` 页面模块：

- `index.tsx`：页面容器。
- 可选小组件：`SelectionText.tsx`，只负责展示原文。

新增或调整权限提示页面模块：

- 页面说明需要开启 macOS Accessibility 权限才能使用划词弹窗。
- 提供“去系统设置”和“稍后”两个操作。
- “去系统设置”调用后端命令打开 Accessibility 设置页。
- “稍后”关闭权限提示窗口。

`App.tsx` 增加路由：

- `/selection-popup` -> `SelectionPopupPage`
- `/accessibility-permission` -> `AccessibilityPermissionPage`

`SelectionPopupPage` 读取 URL query 中的 `text` 参数，解码后展示。若文本为空，显示一个安全空态，但正常情况下后端不会打开空文本窗口。

UI 使用 HeroUI 和现有 Tailwind 体系：

- 使用轻量 Surface/Card 类容器承载文本。
- 文本允许换行，限制最大宽度和最大高度。
- 长单词使用断行策略，避免撑破窗口。

## macOS 权限设计

macOS 上，每次打开应用都检查 Accessibility 权限。

首版要求：

- 如果已授权，启动 `rdev` 划词监听。
- 如果未授权，打开应用内权限提示窗口。
- 权限提示窗口提供跳转系统设置的入口。
- 用户关闭权限提示后，本次运行不再重复提示。
- 未授权时不启动划词监听，不弹 selection popup 空窗口。
- 首版不要求自动检测用户授权后的实时变化；用户可重启应用完成权限状态刷新。

## Windows 权限设计

Windows 首版由 `rdev` 和 `get_selected_text` 工具包处理监听与取词能力。Windows 通常没有类似 macOS Accessibility 的系统授权流程，但仍可能存在读取限制：

- Bugoo 普通权限运行时，可能无法稳定读取管理员权限应用中的选区。
- 部分控件或受保护应用可能不允许读取选中文本。
- UAC 安全桌面和部分受保护应用可能阻止全局监听或选区读取。
- 安全软件拦截全局 hook 时，读取流程会失败。

首版处理方式：

- 读取失败时静默忽略，不打开空窗。
- 记录 debug 或 warn 日志。
- 不要求用户提升管理员权限。

## 数据流

```text
App setup
  -> macOS: check Accessibility permission
  -> missing permission: open accessibility permission window once
  -> granted permission or non-macOS: start rdev listener

用户选中文字并释放鼠标左键
  -> rdev listener receives left button release
  -> get_selected_text reads selected text
  -> empty text: stop, do not open popup
  -> non-empty text: filter length and repeated triggers
  -> accepted text: open or update selection-popup window
  -> frontend page displays selected text only
```

## 错误处理

- Accessibility 权限检查失败：按未授权处理，打开权限提示窗口。
- 权限提示窗口创建失败：记录 warn，应用主功能继续启动。
- 打开系统设置失败：记录 warn，权限窗口保持可见。
- `rdev` 监听注册失败：记录 warn，应用主功能继续运行。
- `get_selected_text` 读取失败：记录 warn，不打开弹窗。
- `get_selected_text` 返回空文本：视为无有效选区，不打开弹窗。
- 窗口创建失败：记录错误，不重试刷屏。

## 测试计划

### Rust 单元测试

覆盖可纯逻辑测试的部分：

- 空文本过滤。
- 超过 50 字符过滤。
- 500ms 节流。
- 短时间重复文本去重。
- URL 参数编码或窗口参数构造。
- 鼠标左键释放会触发读取流程。
- 非鼠标左键释放事件不触发读取流程。
- `get_selected_text` 返回空文本时不打开弹窗。
- macOS 权限缺失时不启动 selection listener，并请求打开权限提示窗口。

Rust 测试按 Rust 模块自身习惯放置，不使用前端 `__test__` 目录规则。

### 前端测试

覆盖 `SelectionPopupPage`：

- query 中有 `text` 时显示原文。
- query 为空时显示安全空态。
- 长文本不会破坏基础布局。

覆盖 `AccessibilityPermissionPage`：

- 显示 Accessibility 权限说明。
- 点击“去系统设置”会调用后端命令。
- 点击“稍后”会关闭权限提示窗口。

前端测试文件放在对应功能模块的 `__test__` 目录下。

### 手动验证

macOS：

- 未授予 Accessibility 权限时，启动应用会出现权限提示窗口。
- 点击“去系统设置”能跳转到 Accessibility 设置页。
- 点击“稍后”关闭后，本次运行不再重复弹出权限提示。
- 未授权时不启动划词弹窗链路，不弹空窗。
- 授权并重启后，在普通文本应用中选中文字会打开浮窗。
- 读取到空选区时不打开浮窗。
- 弹窗只显示选中文字。

Windows：

- 在普通应用中选中文字会打开浮窗。
- 在管理员权限应用或受保护场景中读取失败时不弹空窗。
- 读取到空选区时不打开浮窗。
- 弹窗只显示选中文字。

## 实施边界

本次实现完成后，用户能看到一个“权限检查 -> 划词监听 -> 非空选中文本 -> 浮窗展示原文”的端到端链路。后续翻译、保存、详情、定位优化、关闭策略和更完整的权限状态刷新都作为后续独立迭代处理。
