# Directory Structure

```
src/
├── main.tsx                  # React 入口
├── App.tsx                   # 根组件（路由配置）
├── styles/                   # 全局样式（Tailwind 变量、主题）
├── components/               # 共享 UI 组件
│   └── SvgIcon.tsx
├── hooks/                    # 自定义 React Hooks
│   ├── useWords.ts           # 单词数据获取（React Query）
│   └── useTagSort.ts
├── stores/                   # Zustand 全局状态
│   ├── settingsStore.ts      # 设置缓存
│   └── wordStore.ts          # 单词列表缓存
├── lib/
│   ├── api/                  # Tauri IPC 封装（按模块拆分）
│   │   ├── settings.ts       # 设置读写
│   │   ├── word.ts           # 单词 CRUD
│   │   ├── tags.ts           # 标签管理
│   │   ├── translate.ts      # 翻译
│   │   ├── tts.ts            # 语音合成
│   │   ├── wordDetails.ts    # 单词详情
│   │   └── index.ts          # barrel 导出
│   ├── i18n.ts               # react-i18next 配置
│   └── tagSort.ts            # 工具函数
├── types/                    # TypeScript 类型（与后端 IPC 对齐）
│   └── tag.ts
├── locales/                  # i18n 翻译文件（15 种语言）
│   ├── zh-CN.json
│   ├── en-US.json
│   └── ...
└── pages/                    # 页面组件（按功能组织）
    ├── Home/                 # 首页（单词列表、标签管理）
    │   ├── index.tsx
    │   └── components/       # 页面私有组件
    │       ├── WordList.tsx
    │       ├── WordGrid.tsx
    │       ├── SearchInput.tsx
    │       ├── StatusFilter.tsx
    │       ├── ViewToggle.tsx
    │       ├── BottomBanner.tsx
    │       ├── DetailPanel.tsx
    │       └── TagSection/   # 标签管理子模块
    │           ├── TagSection.tsx
    │           ├── SortableTag.tsx
    │           ├── TagContextMenu.tsx
    │           └── TagEditorDialog.tsx
    ├── Settings/             # 设置页
    │   ├── index.tsx
    │   ├── components/
    │   │   └── SettingItem.tsx  # 通用设置行组件
    │   └── panels/           # 设置面板
    │       ├── ReviewPanel.tsx
    │       ├── GeneralPanel.tsx
    │       ├── TranslationPanel.tsx
    │       ├── NotificationPanel.tsx
    │       ├── AppearancePanel.tsx
    │       ├── ShortcutsPanel.tsx
    │       └── AboutPanel.tsx
    ├── SelectionPopup/       # 划词弹窗
    │   ├── index.tsx
    │   ├── selectionPopupState.ts  # 纯函数状态机
    │   ├── useSelectionPopupResize.ts
    │   ├── Header/
    │   ├── Footer/
    │   └── components/
    ├── AccessibilityPermission/  # 辅助功能权限引导
    └── review/               # 复习模块（规划中）
```

## 文件命名

- 页面目录/组件文件：**PascalCase**（`SelectionPopup`、`WordList.tsx`）
- Hooks：`use` 前缀（`useWords.ts`）
- API 模块：`[name].api.ts` 或 `[name].ts`
- Store 文件：`[name]Store.ts`
- 类型文件：`[name].ts`
- 测试目录：`__test__/`，靠近被测代码

## 组织原则

- **按功能/领域组织**，不按文件类型（如 pages/Home/, pages/Settings/）
- 页面私有组件放在 `components/` 子目录
- 跨页面共享组件放 `src/components/`
- 每个目录用 `index.ts` 做 barrel 导出
