# i18n 多语言切换实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Bugoo 应用引入 react-i18next 多语言基础设施，支持用户切换语言偏好并即时生效。

**Architecture:** 使用 react-i18next 作为 i18n 框架，翻译文件按语言分目录存储（src/locales/{lang}/common.json），语言设置通过 settings.language 持久化。

**Tech Stack:** i18next, react-i18next, i18next-browser-languagedetector

---

## 支持的语言（15种）

| 语言代码 | 名称 | 地区 |
|----------|------|------|
| zh-CN | 简体中文 | 中国大陆 |
| zh-TW | 繁體中文 | 台湾/香港 |
| en | English | 英语（默认） |
| ja | 日本語 | 日语 |
| ko | 한국어 | 韩语 |
| es | Español | 西班牙语 |
| fr | Français | 法语 |
| de | Deutsch | 德语 |
| pt | Português | 葡萄牙语 |
| ru | Русский | 俄语 |
| ar | العربية | 阿拉伯语（RTL） |
| hi | हिन्दी | 印地语 |
| th | ไทย | 泰语 |
| vi | Tiếng Việt | 越南语 |
| id | Bahasa Indonesia | 印尼语 |

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `src/lib/i18n.ts` | 创建 | i18next 初始化配置 |
| `src/locales/zh-CN/common.json` | 创建 | 简体中文翻译 |
| `src/locales/zh-TW/common.json` | 创建 | 繁体中文翻译 |
| `src/locales/en/common.json` | 创建 | 英语翻译 |
| `src/locales/ja/common.json` | 创建 | 日语翻译 |
| `src/locales/ko/common.json` | 创建 | 韩语翻译 |
| `src/locales/es/common.json` | 创建 | 西班牙语翻译 |
| `src/locales/fr/common.json` | 创建 | 法语翻译 |
| `src/locales/de/common.json` | 创建 | 德语翻译 |
| `src/locales/pt/common.json` | 创建 | 葡萄牙语翻译 |
| `src/locales/ru/common.json` | 创建 | 俄语翻译 |
| `src/locales/ar/common.json` | 创建 | 阿拉伯语翻译 |
| `src/locales/hi/common.json` | 创建 | 印地语翻译 |
| `src/locales/th/common.json` | 创建 | 泰语翻译 |
| `src/locales/vi/common.json` | 创建 | 越南语翻译 |
| `src/locales/id/common.json` | 创建 | 印尼语翻译 |
| `src/App.tsx` | 修改 | 启动时初始化 i18n |
| `src/pages/Settings/panels/GeneralPanel.tsx` | 修改 | 语言切换 UI |

---

## 实施步骤

### Task 1: 安装 i18n 依赖

**文件:** 无

- [ ] **Step 1: 安装依赖**

Run: `npm install i18next react-i18next i18next-browser-languagedetector`
Expected: 包安装成功，无报错

---

### Task 2: 创建 i18n 配置文件

**文件:** `src/lib/i18n.ts` (创建)

- [ ] **Step 1: 创建 i18n.ts**

```typescript
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import zhCN from "../locales/zh-CN/common.json";
import zhTW from "../locales/zh-TW/common.json";
import en from "../locales/en/common.json";
import ja from "../locales/ja/common.json";
import ko from "../locales/ko/common.json";
import es from "../locales/es/common.json";
import fr from "../locales/fr/common.json";
import de from "../locales/de/common.json";
import pt from "../locales/pt/common.json";
import ru from "../locales/ru/common.json";
import ar from "../locales/ar/common.json";
import hi from "../locales/hi/common.json";
import th from "../locales/th/common.json";
import vi from "../locales/vi/common.json";
import id from "../locales/id/common.json";

const resources = {
  "zh-CN": { common: zhCN },
  "zh-TW": { common: zhTW },
  en: { common: en },
  ja: { common: ja },
  ko: { common: ko },
  es: { common: es },
  fr: { common: fr },
  de: { common: de },
  pt: { common: pt },
  ru: { common: ru },
  ar: { common: ar },
  hi: { common: hi },
  th: { common: th },
  vi: { common: vi },
  id: { common: id },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: "en",
    defaultNS: "common",
    ns: ["common"],
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ["querystring", "localStorage", "navigator"],
      lookupQuerystring: "lang",
      lookupLocalStorage: "bugoo-language",
    },
  });

export default i18n;
```

---

### Task 3: 创建翻译文件

**文件:** `src/locales/{lang}/common.json` (15个文件)

基础模板（zh-CN）:
```json
{
  "app": {
    "name": "布谷鸟",
    "tagline": "划词翻译与记忆复习"
  },
  "settings": {
    "general": "通用设置",
    "language": "界面语言"
  }
}
```

- [ ] **Step 1: 创建 zh-CN/common.json** (简体中文)
- [ ] **Step 2: 创建 zh-TW/common.json** (繁體中文)
- [ ] **Step 3: 创建 en/common.json** (English)
- [ ] **Step 4: 创建 ja/common.json** (日本語)
- [ ] **Step 5: 创建 ko/common.json** (한국어)
- [ ] **Step 6: 创建 es/common.json** (Español)
- [ ] **Step 7: 创建 fr/common.json** (Français)
- [ ] **Step 8: 创建 de/common.json** (Deutsch)
- [ ] **Step 9: 创建 pt/common.json** (Português)
- [ ] **Step 10: 创建 ru/common.json** (Русский)
- [ ] **Step 11: 创建 ar/common.json** (العربية)
- [ ] **Step 12: 创建 hi/common.json** (हिन्दी)
- [ ] **Step 13: 创建 th/common.json** (ไทย)
- [ ] **Step 14: 创建 vi/common.json** (Tiếng Việt)
- [ ] **Step 15: 创建 id/common.json** (Bahasa Indonesia)

---

### Task 4: 更新 App.tsx

**文件:** `src/App.tsx` (修改)

- [ ] **Step 1: 添加 i18n import**

在文件顶部添加:

```typescript
import "./lib/i18n"; // i18next 初始化
import i18n from "i18next";
```

- [ ] **Step 2: 修改 useEffect 中获取 settings 后设置语言**

当前代码:
```typescript
useEffect(() => {
  seedSettings().then(() => {
    getSettings().then((settings) => {
      setSettings(settings);
    });
  });
}, [setSettings]);
```

修改为:
```typescript
useEffect(() => {
  seedSettings().then(async () => {
    const settings = await getSettings();
    setSettings(settings);
    const lang = settings.language || "en";
    await i18n.changeLanguage(lang);
  });
}, [setSettings]);
```

---

### Task 5: 修改 GeneralPanel 语言切换

**文件:** `src/pages/Settings/panels/GeneralPanel.tsx` (修改)

- [ ] **Step 1: 添加 i18n import**

添加 import:
```typescript
import i18n from "../../../lib/i18n";
```

- [ ] **Step 2: 替换硬编码语言选项为动态列表**

删除硬编码的 `languageOptions`，替换为从 i18n resources 动态获取：

```typescript
// 动态获取 i18n 支持的语言列表
const languageOptions = Object.keys(i18n.options.resources || {}).map((code) => ({
  label: new Intl.DisplayNames([code], { type: "language" }).of(code) || code,
  value: code,
}));
```

- [ ] **Step 3: 添加 changeLanguage 函数**

```typescript
const handleLanguageChange = (newLang: string) => {
  updateSetting("language", newLang);
  i18n.changeLanguage(newLang);
};
```

- [ ] **Step 4: 修改 Select 的 onChange**

```typescript
onChange={(value) => value && handleLanguageChange(String(value))}
```

---

## 语言显示名称映射（用于 Select 选项）

由于 `Intl.DisplayNames` 在某些环境下可能返回 undefined，需要提供 fallback：

```typescript
const languageLabels: Record<string, string> = {
  "zh-CN": "简体中文",
  "zh-TW": "繁體中文",
  en: "English",
  ja: "日本語",
  ko: "한국어",
  es: "Español",
  fr: "Français",
  de: "Deutsch",
  pt: "Português",
  ru: "Русский",
  ar: "العربية",
  hi: "हिन्दी",
  th: "ไทย",
  vi: "Tiếng Việt",
  id: "Bahasa Indonesia",
};

const languageOptions = Object.keys(i18n.options.resources || {}).map((code) => ({
  label: languageLabels[code] || code,
  value: code,
}));
```

---

## 验证方式

1. `npm run dev` 启动应用
2. 打开 DevTools Console，确认无 i18n 初始化错误
3. 打开设置页面 → 通用设置
4. 确认语言下拉列表显示 15 种语言选项
5. 切换不同语言，确认选择后无报错
6. 检查 localStorage 中 `bugoo-language` 键是否被设置
7. 刷新页面，确认语言偏好被保留

---

## 实施检查清单

- [ ] Task 1: 依赖安装成功
- [ ] Task 2: src/lib/i18n.ts 创建完成（15种语言）
- [ ] Task 3: 15个翻译文件创建完成
- [ ] Task 4: App.tsx 更新完成
- [ ] Task 5: GeneralPanel.tsx 改动完成
- [ ] TypeScript 编译无错误: `npx tsc --noEmit`
- [ ] 应用启动无报错
