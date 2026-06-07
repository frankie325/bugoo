# Hook Guidelines

## 数据获取 Hooks

使用 **TanStack React Query**（`@tanstack/react-query`）处理所有服务端数据：

```ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

export function useWords(search?: string) {
  return useQuery({
    queryKey: ['words', search],
    queryFn: async () => {
      const words = await getWords(search);
      return words;
    },
  });
}

export function useAddWord() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: AddWordInput) => addWord(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['words'] });
    },
  });
}
```

**规则**：
- `useQuery` 用于 GET 类操作，`useMutation` 用于写操作
- mutation 成功后调 `queryClient.invalidateQueries` 刷新相关缓存
- queryKey 用数组 `['resource', ...params]` 模式

参考文件：`src/hooks/useWords.ts`

## 自定义 Hooks

命名以 `use` 开头。放在 `src/hooks/` 下。

现有 Hooks：
- `useWords` / `useAddWord` / `useDeleteWord` / `useUpdateWord` — 单词 CRUD（React Query）
- `useTagSort` — 标签排序逻辑
- `useSelectionPopupResize` — 弹窗尺寸自适应

## Tauri 事件监听

用 `listen()` 订阅后端事件，在 `useEffect` cleanup 中取消：

```ts
useEffect(() => {
  let disposed = false;
  let unlisten: (() => void) | undefined;

  listen<string>(TEXT_UPDATED_EVENT, (event) => {
    // handle event
  })
    .then((dispose) => {
      if (disposed) { dispose(); }
      else { unlisten = dispose; }
    })
    .catch((error) => { console.warn("...", error); });

  return () => { disposed = true; unlisten?.(); };
}, []);
```

参考文件：`src/pages/SelectionPopup/index.tsx:274-297`

## 反模式

- **不要**在组件内直接 `invoke()` 获取数据（必须走 `lib/api/` 封装 + React Query）
- **不要**在 `useEffect` 中直接 `invoke()` 而不用 React Query（取消请求、缓存、去重都缺失）
- **不要**创建不做任何封装的一层 wrapper hook
