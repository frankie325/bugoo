# 本地系统 TTS API 设计

## 背景

Bugoo 是一个 Tauri 2 + Rust + React 19 的桌面应用。`AGENTS.md` 已明确把“系统 TTS”列入后端能力，并说明 Rust 后端模块仍在逐步实现。本次只完成本地系统 TTS 的第一段能力：把前端 API、Tauri command 和系统语音命令打通。

当前代码里已经存在并注册了 `commands::tts::speak_text`，但它还是空实现；`src-tauri/src/tts/mod.rs` 也是占位模块。前端 `src/lib/api/` 下已有按功能拆分的 invoke helper，但还没有 TTS helper。

## 目标

- 实现可调用的前端 API：`speakText(text, lang?)`。
- 实现已注册的 Tauri command：`speak_text`。
- 使用系统自带 TTS，不接入网络服务，不需要 API Key。
- 播放启动后尽快返回，避免长文本朗读阻塞调用方。
- 为后续详情页播放按钮、自动朗读和云端 TTS 留出清晰边界。

## 非目标

- 不新增任何 UI。
- 不修改单词详情、卡片、列表或设置页。
- 不接入现有 `autoSpeak` 设置。
- 不实现语音列表、音量、语速、音调配置。
- 不生成或缓存音频文件，不写入 `audio_url`。
- 不接入云端 TTS。

## 方案

采用轻量的系统命令封装：

- macOS 使用 `say`。
- Windows 使用 PowerShell 调用 `System.Speech.Synthesis.SpeechSynthesizer`。
- Linux 使用 `spd-say`。

Rust 侧使用 `Command::spawn` 启动朗读进程，不等待播放完成。这样 Tauri command 只表示“系统 TTS 已经开始启动”，不承担播放完成、取消播放或播放状态追踪。

## 架构

前端新增：

```ts
speakText(text: string, lang?: string): Promise<void>
```

它位于 `src/lib/api/tts.ts`，并从 `src/lib/api/index.ts` 重新导出。调用方式保持项目现有 API 风格：

```ts
invoke("speak_text", { text, lang })
```

后端分两层：

- `src-tauri/src/commands/tts.rs`：Tauri command 边界，只接收参数并委托给 TTS 模块。
- `src-tauri/src/tts/mod.rs`：平台相关实现，负责文本清洗、语言提示处理、选择系统命令和启动进程。

`src-tauri/src/lib.rs` 已经注册 `commands::tts::speak_text`，本次无需改动注册列表，除非实现时发现签名需要和 Tauri 参数保持一致。

## 数据流

1. 调用方执行 `speakText(text, lang?)`。
2. 前端 helper 调用 Tauri `invoke("speak_text", { text, lang })`。
3. `commands::tts::speak_text` 将请求交给 `crate::tts::speak_text`。
4. TTS 模块 trim 文本；空文本直接成功返回。
5. 非空文本按操作系统构造命令并 `spawn`。
6. 命令启动失败时返回错误字符串给前端。

## 语言处理

`lang` 是可选的轻量提示，不保证跨平台精确选中同一声音。

- macOS 可以对常见中文提示如 `zh`、`zh-CN` 尝试使用中文系统语音。
- 英文或未知语言优先使用系统默认语音。
- Windows 和 Linux 第一版可以忽略 `lang`，交给系统默认语音。
- 如果指定语音不可用，应回退默认语音，而不是让播放失败。

## 错误处理

- 空文本或只有空白字符：返回成功。
- 系统命令不存在或启动失败：返回可读错误。
- 子进程启动后的播放失败：本阶段不追踪。
- Linux 缺少 `spd-say`：返回错误，未来 UI 可以提示用户安装 speech-dispatcher。

## 文件变更范围

- `src-tauri/src/tts/mod.rs`：实现本地系统 TTS。
- `src-tauri/src/commands/tts.rs`：从空实现改为调用 TTS 模块。
- `src/lib/api/tts.ts`：新增前端 API helper。
- `src/lib/api/index.ts`：导出 `speakText`。

本次不修改数据库、设置页、Home 页面或 HeroUI 组件。

## 验证

按 `AGENTS.md` 中的项目命令验证：

- `npm run build`
- `cd src-tauri && cargo build`
- `cd src-tauri && cargo test`

如果本机环境没有 `npm`，可以用已安装的包管理器等价执行前端构建；最终仍以 TypeScript/Vite 构建通过为准。

手动验证可以在开发环境里临时调用 `speakText("hello", "en")` 和 `speakText("你好", "zh-CN")`。临时代码不提交。

## 后续扩展

- 在单词详情面板增加播放按钮。
- 在卡片或列表项增加可选播放入口。
- 将 `autoSpeak` 接入翻译、复习或详情打开流程。
- 增加语音、语速、音量设置。
- 增加云端 TTS provider，并让本地系统 TTS 作为默认离线方案。
