# Component Guidelines

## 组件结构

所有组件使用**函数声明**（非箭头函数），Props 用 `interface` 定义：

```tsx
interface WordListProps {
  words: Word[];
  onWordClick: (word: Word) => void;
}

export default function WordList({ words, onWordClick }: WordListProps) {
  return (
    <div className="flex flex-col gap-2">
      {words.map((word) => (
        <div key={word.id} onClick={() => onWordClick(word)}>
          ...
        </div>
      ))}
    </div>
  );
}
```

**不要使用 `React.FC`**，始终用显式 Props 解构。

参考文件：`src/pages/Home/components/WordList.tsx`

## HeroUI v3 组件

优先使用 HeroUI 组件而非自定义样式：

```tsx
import { Card, Switch, RadioGroup, Radio, Separator, NumberField } from "@heroui/react";
```

- `Card` / `Card.Header` / `Card.Content` / `Card.Title` 用于分区卡片
- `Switch` + `Switch.Control` + `Switch.Thumb` 用于开关（**不用** `Switch.Label`）
- `RadioGroup` + `Radio` + `Radio.Control` + `Radio.Indicator` 用于单选
- `NumberField` + `NumberField.Group` + `NumberField.Input` + `NumberField.DecrementButton` / `IncrementButton` 用于数字输入
- `Separator` 分隔设置项

参考文件：`src/pages/Settings/panels/ReviewPanel.tsx`

## 颜色变量

严格使用 HeroUI 和项目已定义的 CSS 变量，**禁止硬编码 `#[hex]`**：

- 背景：`bg-background`、`bg-surface`
- 文字：`text-foreground`、`text-muted`
- 边框：`border-border`、`border-divider`
- 品牌色：`text-accent`、`bg-accent`、`bg-accent-soft`
- 项目品牌绿阶：`accent-1` ~ `accent-10`（来自 `src/styles/variable.css`）

## Props 约定

- 用 `interface` 定义，不用 `type`
- 回调 Props 显式类型：`onSelect: (id: string) => void`
- service 层数据用 API 类型（`Word`、`TagItem` 等），不要在内联 props 里重复定义字段

## 事件处理

用 `useCallback` 包装传递给子组件的回调，避免不必要的 re-render：

```tsx
const handleAddWord = useCallback(async () => {
  // ...
}, [resolvedWord, selectedTags]);
```

参考文件：`src/pages/SelectionPopup/index.tsx`

## 条件渲染

用三元或 `&&`，不用早期 return 在 JSX 中间打断：

```tsx
{selectionLimitEnabled && (
  <NumberField value={maxSelectionChars} ...>
    ...
  </NumberField>
)}
```

## 反模式

- **不要**用 `div` 栈代替 Card 等语义组件
- **不要**内联 `style={{}}` 硬编码颜色
- **不要**在组件内定义新组件（破坏 memo）
- **不要**将 HeroUI 组件与自定义样式混合（保持用 HeroUI API）
- **不要**用 `React.FC`
