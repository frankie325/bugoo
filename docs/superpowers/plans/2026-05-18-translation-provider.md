# Translation Provider 功能实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task.

日期：2026-05-18  
状态：已重建（根据 `docs/superpowers/specs/2026-05-18-translation-provider-design.md`）

---

## 目标

实现统一翻译能力和单词详情生成能力，支持：

- 多引擎翻译入口（`openai` / `custom` / `deepl` / `google`）
- 划词翻译返回示例句摘要
- 单词详情页展示释义、例句、记忆技巧、详情说明
- 设置页可配置引擎、模型、Prompt、超时
- SQLite 持久化详情数据，避免重复生成

---

## 实施范围

### 包含

- Rust 端口定义与 adapter 实现
- Tauri commands（翻译、详情、设置默认值）
- SQLite `word_details` 表与迁移
- 前端 API 类型扩展
- 设置页翻译面板扩展
- 详情面板生成/加载逻辑与错误处理
- 多语言文案补齐

### 不包含

- 全局划词监听完整链路
- DeepL / Google 第一版完整云端接入
- 翻译用量计费与统计

---

## 架构与目录

```txt
src-tauri/src/
├── commands/
│   ├── translate.rs
│   └── word_details.rs
├── ports/outbound/
│   ├── translation.rs
│   └── word_insight.rs
└── adapters/outbound/translation/
    ├── mod.rs
    ├── openai.rs
    ├── custom.rs
    ├── deepl.rs
    ├── google.rs
    └── prompts/
        ├── system_translation_prompt.txt
        └── word_detail_prompt.txt
```

---

## 执行任务

### Task 1: 定义翻译与单词详情端口

**文件：**

- `src-tauri/src/ports/outbound/translation.rs`（新增）
- `src-tauri/src/ports/outbound/word_insight.rs`（新增）
- `src-tauri/src/ports/outbound/mod.rs`（修改）

- [ ] Step 1: 新增 `TranslationProvider`、`TranslationConfig`、`TranslationResult`、`TranslationError`、`TranslationExample` 定义。
- [ ] Step 2: 新增 `WordInsightProvider`、`GeneratedWordDetail`、`WordInsightRequest` 定义。
- [ ] Step 3: 在 `ports/outbound/mod.rs` 导出 `translation` 和 `word_insight`。
- [ ] Step 4: 通过 `cargo test ports::outbound` 做最小可用验证。

**验收：**

- 端口类型可被 commands 和 adapters 引用。
- `TranslationConfig` 包含：`engine/api_endpoint/api_key/translation_model/translation_prompt/word_detail_prompt/timeout_ms`。

---

### Task 2: 增加 `word_details` 表与迁移

**文件：**

- `src-tauri/src/db/migrations.rs`（修改）
- `src-tauri/src/db/mod.rs`（修改）
- `src-tauri/src/adapters/outbound/sqlite.rs`（修改）

- [ ] Step 1: 在迁移中新增 `word_details` 表（JSON 字段 + `ON DELETE CASCADE`）。
- [ ] Step 2: `raw_json` 默认值使用合法 JSON（`{}`），避免空串。
- [ ] Step 3: 启用 SQLite 外键：`PRAGMA foreign_keys = ON`。
- [ ] Step 4: 删除单词时，确保详情记录与复习记录不会残留。
- [ ] Step 5: 运行 `cargo build` 验证迁移和数据层。

**验收：**

- 数据库升级后可查询/写入 `word_details`。
- 删除 `words` 记录后不存在孤儿详情数据。

---

### Task 3: 实现 OpenAI-compatible 翻译 adapter

**文件：**

- `src-tauri/src/adapters/outbound/translation/mod.rs`（新增）
- `src-tauri/src/adapters/outbound/translation/openai.rs`（新增）
- `src-tauri/src/adapters/outbound/translation/custom.rs`（新增）
- `src-tauri/src/adapters/outbound/translation/deepl.rs`（新增）
- `src-tauri/src/adapters/outbound/translation/google.rs`（新增）
- `src-tauri/src/adapters/outbound/translation/prompts/system_translation_prompt.txt`（新增）
- `src-tauri/src/adapters/outbound/translation/prompts/word_detail_prompt.txt`（新增）
- `src-tauri/src/adapters/outbound/mod.rs`（修改）

- [ ] Step 1: 新增 `openai.rs`，实现 `TranslationProvider` + `WordInsightProvider`。
- [ ] Step 2: 系统翻译 Prompt、详情 Prompt 外置为文件，禁止硬编码在 Rust。
- [ ] Step 3: `custom` 复用 OpenAI-compatible 协议实现。
- [ ] Step 4: `deepl/google` 第一版返回明确 `UnsupportedEngine` 错误。
- [ ] Step 5: HTTP/JSON 错误处理补齐，API key 在错误日志中脱敏。
- [ ] Step 6: 增加解析单测：成功、无效 JSON、空字段、错误信息提取、URL 拼接。

**验收：**

- `cargo test adapters::outbound::translation::openai` 通过。
- 在 `openai/custom` 引擎下可生成 `translation + examples + detail`。

---

### Task 4: 实现翻译与详情 Tauri 命令

**文件：**

- `src-tauri/src/commands/translate.rs`（新增/修改）
- `src-tauri/src/commands/word_details.rs`（新增）
- `src-tauri/src/commands/mod.rs`（修改）
- `src-tauri/src/lib.rs`（修改）

- [ ] Step 1: `translate_text` 命令：读取 settings -> 构建 config -> 选择 provider -> 返回统一结果。
- [ ] Step 2: `word_details` 命令：
  - `get_word_detail`
  - `generate_word_detail`
  - `save_word_detail`
- [ ] Step 3: 确保 async await 期间不长期持有 DB guard。
- [ ] Step 4: 在 `lib.rs` 注册所有新增 invoke 命令。
- [ ] Step 5: `cargo test` 与 `cargo build` 验证。

**验收：**

- 前端 `invoke("translate_text")` 可返回统一字段。
- `generate_word_detail` 可落盘并被 `get_word_detail` 读取。

---

### Task 5: 扩展 settings 默认值与翻译设置面板

**文件：**

- `src-tauri/src/commands/settings.rs`（修改）
- `src/pages/Settings/panels/TranslationPanel.tsx`（修改）
- `src/locales/*/common.json`（修改）

- [ ] Step 1: 在 `seed_settings` 增加字段并使用 `INSERT OR IGNORE`：
  - `translationEngine`
  - `apiEndpoint`
  - `apiKey`
  - `translationModel`
  - `translationPrompt`
  - `wordDetailPrompt`
  - `translationTimeoutMs`
- [ ] Step 2: 设置页翻译面板增加上述配置项 UI。
- [ ] Step 3: Prompt 相关输入使用 `TextArea`，超时使用 `NumberField`（HeroUI v3 结构）。
- [ ] Step 4: 所有语言包补齐对应文案 key。

**验收：**

- 首次启动后 settings 表自动补齐默认值，不覆盖已有值。
- 面板字段变更后可持久化到 SQLite。

---

### Task 6: 扩展前端 API 类型

**文件：**

- `src/lib/api/translate.ts`（修改）
- `src/lib/api/wordDetails.ts`（新增）
- `src/lib/api/index.ts`（修改）

- [ ] Step 1: `translate.ts` 统一 snake_case -> camelCase 映射并加数组字段防御。
- [ ] Step 2: 新增 `wordDetails.ts`（`getWordDetail` / `generateWordDetail` + 类型映射）。
- [ ] Step 3: `index.ts` 导出新增 API 和类型。
- [ ] Step 4: `pnpm tsc --noEmit` 验证。

**验收：**

- 前端使用端无需关心 Rust 字段命名差异。
- 空数组/异常数据不导致 UI 崩溃。

---

### Task 7: 首页详情展示与交互完善

**文件：**

- `src/pages/Home/components/DetailPanel.tsx`（修改）
- `src/locales/*/common.json`（修改）

- [ ] Step 1: 打开详情面板时加载 `getWordDetail(word.id)`。
- [ ] Step 2: 无详情时显示“生成详情”按钮，调用 `generateWordDetail(word.id)`。
- [ ] Step 3: 展示区块：
  - 释义
  - 例句
  - 记忆技巧
  - 详情说明
- [ ] Step 4: 增加加载失败提示与“重试加载”入口。
- [ ] Step 5: 修复并发竞态：切词/关闭后旧请求不得回写当前状态。
- [ ] Step 6: 文案多语言补齐（含 `retryLoadDetail`）。

**验收：**

- 不出现旧请求覆盖新单词详情问题。
- 失败态可重试读取而不是只能重新生成。

---

### Task 8: 总体验证与交付

**命令：**

- [ ] Step 1: `pnpm tsc --noEmit`
- [ ] Step 2: `pnpm build`
- [ ] Step 3: `cd src-tauri && cargo build`
- [ ] Step 4: `cd src-tauri && cargo test`

**手动检查：**

- [ ] 设置页翻译配置项可读写并持久化。
- [ ] 单词详情“生成 -> 保存 -> 再次打开可读取”链路可用。
- [ ] 多语言切换后详情面板文案不混杂英文。

---

## 风险与回滚策略

### 风险点

- 外部翻译 API 返回格式不稳定。
- 配置缺失导致请求失败。
- 详情生成响应慢，影响交互感受。

### 防护

- 对 JSON 解析进行严格校验与降级处理。
- settings 提供默认值，且仅缺失时补齐。
- UI 增加 loading/error/retry 明确状态。

### 回滚策略

- provider 异常时保持基础翻译功能可用（不阻塞主流程）。
- 详情生成失败不影响词条列表和复习流程。

---

## 备注

- 本文为计划文档重建版，路径与原计划保持一致。  
- 若后续需要严格字节级还原，请从你本机 Time Machine / IDE Local History / 云端同步历史中提取原文件再覆盖当前版本。
