# Quality Guidelines

## 禁止模式

- **硬编码颜色**：禁止 `text-[#hex]` / `bg-[#hex]` / `border-[#hex]`。只用 HeroUI CSS 变量和项目 `variable.css` 中定义的 token
- **`any` 类型**：禁止使用 `any`。用 `unknown` + 类型守卫替代
- **`console.log`**：禁止在 production 代码中留 `console.log`
- **`dangerouslySetInnerHTML`**：除非有 XSS 消毒，禁止
- **`React.FC`**：不用 `React.FC`，用显式 Props 类型
- **`style={{}}` 内联样式**：优先 Tailwind 工具类
- **HeroUI 不存在的样式**：不用 `text-default-50`、`bg-primary`、`text-primary` 等 HeroUI v3 不存在的 token

## 测试要求

- 测试文件放在被测代码同目录的 `__test__/` 下
- 用 Vitest + React Testing Library
- 测试命名：描述预期行为（`returns empty array when no markets match query`）

```bash
pnpm test                    # 跑全部
pnpm test -- src/pages/...  # 按路径过滤
pnpm test:watch              # watch 模式
```

参考文件：`src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`

## 国际化 (i18n)

使用 `react-i18next`，所有用户可见文案用 `t("key")`：

```tsx
const { t } = useTranslation();
<Card.Title>{t("settings.review.title")}</Card.Title>
```

新增文案需要覆盖 `src/locales/` 下所有 15 个语言文件。

## API 调用规范

所有 Tauri IPC 调用封装在 `src/lib/api/` 下，不在组件中直接 `invoke`：

```ts
// src/lib/api/settings.ts
export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}
```

## 文件大小

- 组件文件：< 400 行
- API 模块：< 100 行
- Store 文件：< 80 行

## 反模式

- **不要**在组件中直接 `import { invoke } from "@tauri-apps/api/core"`
- **不要**跳过 i18n 硬编码中文/英文字符串
- **不要**跳过 `lib/api/` 封装层直接调 IPC
