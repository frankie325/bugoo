# State Management

## 状态分类

| 类型 | 工具 | 示例 |
|------|------|------|
| 服务端数据 | React Query (`@tanstack/react-query`) | 单词列表、标签 |
| 客户端全局状态 | Zustand | 设置缓存、UI 状态 |
| 本地组件状态 | `useState` / `useRef` | 表单输入、弹窗可见性 |
| URL 状态 | React Router `useLocation` / `useSearchParams` | 当前页面、选中文本 |

## Zustand Store 模式

```ts
import { create } from "zustand";

interface SettingsState {
  settings: Record<string, string>;
  setSettings: (settings: Record<string, string>) => void;
  updateSetting: (key: string, value: string) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: {},
  setSettings: (settings) => set({ settings }),
  updateSetting: (key, value) => {
    set((state) => ({
      settings: { ...state.settings, [key]: value },
    }));
    return setSetting(key, value).catch((error) => {
      console.warn(`Failed to persist setting ${key}:`, error);
    });
  },
}));
```

**规则**：
- `updateSetting` 先乐观更新本地状态，再异步持久化到后端
- 后端持久化失败只 `console.warn`，不回滚 UI（乐观更新）
- Store 内部不可变更新：`{ ...state.settings, [key]: value }`

参考文件：`src/stores/settingsStore.ts`、`src/stores/wordStore.ts`

## React Query + Zustand 协同

React Query 获取数据后同步到 Zustand store 供全局消费：

```ts
const setWords = useWordStore((state) => state.setWords);

return useQuery({
  queryKey: ['words', search],
  queryFn: async () => {
    const words = await getWords(search);
    setWords(words);  // 同步到 Zustand
    return words;
  },
});
```

## 本地状态

只在组件内使用的状态用 `useState`：
- 表单输入值
- UI 开关状态
- 加载/错误标记

## 反模式

- **不要**把服务端数据存两份（React Query cache + Zustand store 里重复完整数据）
- **不要**在 React Query `queryFn` 外手动调用 `invoke` 更新 store
- **不要**用全局状态管理不需要共享的本地 UI 状态
