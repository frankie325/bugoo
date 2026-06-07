# 颜色系统

> 绘制页面时颜色**必须**使用以下三种来源，按优先级排序。

## 优先级

1. **HeroUI v3 已定义的 CSS 变量**（参考 `https://github.com/heroui-inc/heroui/blob/v3/packages/styles/themes/shared/theme.css`）
2. **项目自定义变量**（`src/styles/variable.css` 中 `@theme inline` 导出的 token）
3. **Tailwind 工具类**（如 `bg-white`、`text-black`）—— 仅在 1+2 都不满足时使用

**严禁**使用 `text-[#hex]` / `bg-[#hex]` / `border-[#hex]` 等硬编码 hex 颜色。

---

## HeroUI v3 颜色变量清单

来源：`@theme inline { --color-*: ... }` 在 `packages/styles/themes/shared/theme.css` 中定义。

| CSS 变量 | 用途 | 生成的 Tailwind 类 |
|---------|------|---------------------|
| `--color-background` | 应用主背景 | `bg-background` |
| `--color-foreground` | 应用主前景文字 | `text-foreground` |
| `--color-surface` | 卡片/面板背景 | `bg-surface` |
| `--color-surface-foreground` | 卡片上的文字 | `text-surface-foreground` |
| `--color-surface-hover` | 卡片 hover | `hover:bg-surface-hover` |
| `--color-surface-secondary` | 次级面板（如 input） | `bg-surface-secondary` |
| `--color-surface-secondary-foreground` | 次级面板上的文字 | `text-surface-secondary-foreground` |
| `--color-surface-tertiary` | 三级面板 | `bg-surface-tertiary` |
| `--color-surface-tertiary-foreground` | 三级面板上的文字 | `text-surface-tertiary-foreground` |
| `--color-overlay` | 浮层（popover/dropdown）背景 | `bg-overlay` |
| `--color-overlay-foreground` | 浮层文字 | `text-overlay-foreground` |
| `--color-muted` | 静音/次要内容 | `bg-muted` / `text-muted` |
| `--color-accent` | 品牌主色 | `bg-accent` / `text-accent` |
| `--color-accent-foreground` | 品牌主色上的文字 | `text-accent-foreground` |
| `--color-accent-soft` | 品牌主色软底 | `bg-accent-soft` |
| `--color-accent-soft-foreground` | 软底上的文字 | `text-accent-soft-foreground` |
| `--color-segment` | 分段控件 | `bg-segment` / `text-segment-foreground` |
| `--color-border` | 边框 | `border-border` |
| `--color-border-secondary` | 次级边框 | `border-border-secondary` |
| `--color-border-tertiary` | 三级边框 | `border-border-tertiary` |
| `--color-separator` | 分隔线 | `bg-separator` |
| `--color-separator-secondary` | 次级分隔线 | `bg-separator-secondary` |
| `--color-separator-tertiary` | 三级分隔线 | `bg-separator-tertiary` |
| `--color-focus` | 聚焦环 | `ring-focus` |
| `--color-link` | 链接 | `text-link` |
| `--color-default` | 中性默认（按钮等） | `bg-default` |
| `--color-default-foreground` | 中性默认上的文字 | `text-default-foreground` |
| `--color-default-hover` | 中性默认 hover | `hover:bg-default-hover` |
| `--color-default-soft` | 中性软底 | `bg-default-soft` |
| `--color-default-soft-foreground` | 中性软底文字 | `text-default-soft-foreground` |
| `--color-default-soft-hover` | 中性软底 hover | `hover:bg-default-soft-hover` |
| `--color-success` / `-hover` | 成功状态 | `bg-success` / `text-success` |
| `--color-success-foreground` | 成功状态上的文字 | `text-success-foreground` |
| `--color-success-soft` / `-foreground` / `-hover` | 成功软底 | `bg-success-soft` / `text-success-soft-foreground` |
| `--color-warning` / `-hover` / `-foreground` | 警告状态 | `bg-warning` / `text-warning` |
| `--color-warning-soft` / `-foreground` / `-hover` | 警告软底 | `bg-warning-soft` |
| `--color-danger` / `-hover` / `-foreground` | 错误状态 | `bg-danger` / `text-danger` |
| `--color-danger-soft` / `-foreground` / `-hover` | 错误软底 | `bg-danger-soft` |
| `--color-backdrop` | 模态背景遮罩 | `bg-backdrop` |

---

## HeroUI v3 中**不存在**的样式

下列样式名在 HeroUI v3 中不存在，使用无效果：

- ~~`text-default-400/500/600/700/900`~~ — 缺少数字阶 token
- ~~`bg-default-50/100/200`~~ — 缺少数字阶 token
- ~~`text-primary` / `bg-primary` / `border-primary`~~
- ~~`text-accent-1/2/.../10`~~ — 项目自定义了 `accent-1`~`accent-10` 数字阶，与 HeroUI 的 `--color-accent` 语义不同

---

## 项目自定义变量

`src/styles/variable.css` 通过 `@theme { --color-accent-N: ... }` 导出品牌绿色阶（仅品牌主色相关）：

| CSS 变量 | 来源 | 用途 |
|---------|------|------|
| `--color-accent-1` ~ `--color-accent-10` | 项目 variable.css | 品牌绿色阶（light/dark 各 10 阶） |
| `--accent` (= `--accent-6`) | 项目 variable.css | 品牌主色 alias |

**注意：** 项目自定义的 `accent-1`~`accent-10` 与 HeroUI 的 `--color-accent`（单一品牌色）不同名空间。需要品牌色阶对比时用 `accent-N`；需要单一品牌色（如按钮、强调文字）时用 HeroUI 的 `text-accent` / `bg-accent`。

参考文件：`src/styles/variable.css`

---

## 添加新颜色变量

如果 HeroUI v3 和项目 variable.css 都不满足需求：

1. **先在 HeroUI 的 theme.css 中找最接近的语义 token** — 优先复用 `default` / `surface` / `border` / `muted` 的不同层级（`-hover` / `-soft` / `-foreground` / `-secondary` / `-tertiary`）
2. **如果确实需要新的色阶**，在 `src/styles/variable.css` 中按 Tailwind v4 `@theme { --color-xxx-N: ... }` 格式定义：
   ```css
   @theme {
     --color-my-purpose-50: #fafafa;
     --color-my-purpose-100: #f5f5f5;
     /* ... */
   }
   ```
3. 通过 `bg-my-purpose-50` / `text-my-purpose-100` 等 Tailwind 工具类使用

---

## 验证清单

- [ ] 不用 `text-[#xxx]` / `bg-[#xxx]` / `border-[#xxx]`
- [ ] 不使用 HeroUI v3 中不存在的 token（见上方清单）
- [ ] 新引入的颜色先在 `.heroui-docs/react/` 中搜索 `bg-` / `text-` 找 HeroUI 是否有类似语义
- [ ] 真正的自定义色阶追加到 `variable.css` 而不是直接硬编码
- [ ] 在 light/dark 主题下都验证颜色对比度（HeroUI 的语义 token 已自动适配）
