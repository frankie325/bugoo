# i18n 多语言切换设计

## Context

为 Bugoo 应用引入多语言支持基础设施。第一阶段聚焦于语言切换功能本身（保存用户语言偏好到 settings 表，即时切换 UI 语言），不包含 UI 文字翻译。翻译文件在后续阶段按需添加。

## 架构

- **库**: `react-i18next` + `i18next`
- **翻译文件**: `src/locales/{lang}/common.json`（如 `zh-CN/common.json`、`en/common.json`）
- **i18n 初始化**: `src/lib/i18n.ts`
- **语言存储**: 通过 `settings.language` 字段（已存在）

## 目录结构

```
src/
├── locales/
│   ├── zh-CN/common.json
│   ├── en/common.json
│   ├── zh-TW/common.json
│   └── ja/common.json
├── lib/
│   └── i18n.ts          # i18next 初始化配置
├── App.tsx              # 启动时初始化 i18n 语言
├── pages/Settings/panels/GeneralPanel.tsx  # 语言选择器
└── components/           # 后续使用 t() 函数翻译 UI
```

## 数据流

### 1. 初始化流程（App.tsx）

```
应用启动
  → seedSettings() + getSettings()
  → 读取 settings.language
  → i18n.init({ lng: settings.language })
```

### 2. 语言切换流程（GeneralPanel）

```
用户选择语言
  → updateSetting("language", selectedLang)
  → i18n.changeLanguage(selectedLang)
  → UI 即时更新
```

### 3. 翻译文件格式

`src/locales/zh-CN/common.json` 示例：

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

## 实现步骤

### Step 1: 安装依赖

```bash
npm install i18next react-i18next i18next-browser-languagedetector
```

### Step 2: 创建 i18n 配置文件

**文件**: `src/lib/i18n.ts`

```typescript
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import zhCN from "../locales/zh-CN/common.json";
import en from "../locales/en/common.json";
import zhTW from "../locales/zh-TW/common.json";
import ja from "../locales/ja/common.json";

const resources = {
  "zh-CN": { common: zhCN },
  en: { common: en },
  "zh-TW": { common: zhTW },
  ja: { common: ja },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: "zh-CN",
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

### Step 3: 更新 App.tsx

在应用启动时初始化 i18n 并从 settings 恢复语言设置：

```typescript
import "./lib/i18n"; // 初始化 i18next
import { getSettings } from "./lib/api";
import { useSettingsStore } from "./stores/settingsStore";

// useEffect 中:
const settings = await getSettings();
const lang = settings.language || "zh-CN";
await i18n.changeLanguage(lang);
```

### Step 4: 创建翻译文件

创建以下文件（暂用英文 key，后续填充翻译）：

- `src/locales/zh-CN/common.json`
- `src/locales/en/common.json`
- `src/locales/zh-TW/common.json`
- `src/locales/ja/common.json`

### Step 5: 更新 GeneralPanel 语言选择

修改 `GeneralPanel.tsx`，将语言选项从硬编码改为动态从 i18n 可用语言列表获取，并添加 `i18n.changeLanguage()` 调用：

```typescript
import { useTranslation } from "react-i18next";
import i18n from "../../lib/i18n";

// 获取 i18n 支持的语言列表
const languageOptions = Object.entries(i18n.options.resources || {}).map(
  ([code, ns]) => ({
    label: new Intl.DisplayNames([code], { type: "language" }).of(code) || code,
    value: code,
  })
);

// 切换语言
const handleLanguageChange = (newLang: string) => {
  updateSetting("language", newLang);
  i18n.changeLanguage(newLang);
};
```

## 依赖

| 包 | 用途 |
|----|------|
| i18next | 核心 i18n 框架 |
| react-i18next | React 绑定 |
| i18next-browser-languagedetector | 自动检测浏览器语言 |

## 验证方式

1. `npm run dev` 启动应用
2. 打开设置页面，确认语言选择器工作
3. 选择不同语言，确认 `i18n.language` 更新
4. 检查 localStorage 中 `bugoo-language` 是否保存
5. 刷新页面，确认语言偏好被保留

## 后续扩展

- UI 组件使用 `t("key.path")` 函数替换硬编码文字
- 添加新语言只需在 `locales/{lang}/common.json` 添加翻译文件
- 语言列表可从 `i18n.options.resources` 动态读取
