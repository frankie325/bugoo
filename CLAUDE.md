# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bugoo (布谷鸟) — a cross-platform desktop app for word selection translation with Ebbinghaus spaced repetition memory review. Core features: select text anywhere to translate, save words to word list, receive review notifications based on spaced repetition algorithm.

## Tech Stack

- **Backend**: Tauri 2.x with Rust (SQLite via rusqlite, DeepL API, system TTS)
- **Frontend**: React 18 + TypeScript (Vite)
- **Database**: SQLite with SM-2 spaced repetition algorithm
- **Translation**: DeepL API
- **Notifications**: System notifications via Tauri

## Project Structure

Tauri 2.x standard layout at project root:

```
bugoo/
├── package.json              # Frontend dependencies
├── index.html               # Frontend entry
├── vite.config.ts           # Vite config
├── tsconfig.json            # TypeScript config
├── src/                     # Frontend source (React/TS)
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── FloatWindow.tsx       # Translation popup
│   │   ├── WordList.tsx          # Word list
│   │   ├── WordDetail.tsx        # Word detail card
│   │   └── ReviewNotification.tsx # Review notification
│   ├── hooks/
│   ├── styles/
│   └── lib/
├── src-tauri/               # Tauri application
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/         # Tauri commands (translate, words, review, tts)
│   │   ├── db/               # SQLite (mod, migrations, models)
│   │   ├── scheduler/         # SM-2 algorithm + notification scheduler
│   │   └── tts/              # TTS (macOS say, Windows SAPI)
│   ├── icons/
│   └── capabilities/
└── docs/superpowers/
    ├── plans/                # Implementation plans
    └── design/               # Design specs
```

## Common Commands

```bash
# Frontend dev
npm run dev

# Tauri dev (full app)
npm run tauri dev

# Build
npm run tauri build

# Rust only
cargo build
cargo test
```

## Architecture Notes

- `src-tauri/src/lib.rs` — Tauri builder, module initialization
- `src-tauri/src/commands/` — Tauri commands exposed to frontend (translate, words CRUD, review, tts)
- `src-tauri/src/db/` — SQLite connection, migrations, models
- `src-tauri/src/scheduler/ebbinghaus.rs` — SM-2 algorithm implementation
- `src-tauri/src/scheduler/notification.rs` — Review notification scheduler
- `src-tauri/src/tts/` — Cross-platform TTS (macOS/Windows)

## Superpowers Workflow

Plans in `docs/superpowers/plans/`, designs in `docs/superpowers/design/`.
