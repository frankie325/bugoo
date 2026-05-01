# CLAUDE.md

本文件为 Claude Code（claude.ai/code）在处理本代码库时提供指导。

## 项目概述

Bugoo（布谷鸟）— 跨平台桌面应用，实现划词翻译和艾宾浩斯遗忘曲线记忆复习。核心功能：全局选中文字即可翻译、保存生词到生词本、根据间隔重复算法接收复习提醒通知。

## 技术栈

- **后端**：Tauri 2.x + Rust（rusqlite 操作 SQLite、DeepL API、系统 TTS）
- **前端**：React 19 + TypeScript（Vite）+ HeroUI + Tailwind CSS 4
- **数据库**：SQLite + SM-2 间隔重复算法
- **翻译**：DeepL API
- **通知**：通过 Tauri 发送系统通知

## 常用命令

```bash
# 前端开发
npm run dev

# Tauri 开发（完整应用）
npm run tauri dev

# 构建 Tauri 应用
npm run tauri build

# Rust 单独构建/测试
cd src-tauri && cargo build
cd src-tauri && cargo test
```

## 架构说明
参考 @docs/architecture.md 文档

### 前端 (`src/`)
- `App.tsx` — 根组件
- `lib/api.ts` — Tauri invoke 调用封装（translate、addWord、getWords、deleteWord）
- `hooks/` — React 自定义 hooks
- `stores/` — 状态管理
- `types/` — TypeScript 类型定义

### Tauri 后端 (`src-tauri/src/`)
- `main.rs` — 应用入口
- **待实现的命令模块：**
- **待实现的数据库模块：** 
- **待实现的调度模块：** 
- **待实现的 TTS 模块：**

**注意：** Rust 后端大部分模块尚为空实现，需要按模块逐步开发。
