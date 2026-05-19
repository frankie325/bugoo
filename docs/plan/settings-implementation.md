# 设置页面功能实现计划

## Context

用户要求根据 `docs/setting.md` 实现完整的设置页面功能。需求包括：通用设置、翻译设置、学习设置、通知设置、数据与同步、外观与个性化、快捷键、关于等模块。需要设计设置页的打开动画，并确定配置信息的存储方案。

**用户明确要求：Settings 应该是页面（Page），而不是抽屉组件（Drawer）。**

## 存储方案决策

**推荐使用 SQLite 存储设置**，原因：
1. 后端 Rust 可直接访问，与 words/review 数据存储一致
2. 支持复杂数据结构（如数组、对象）
3. 便于后续云同步功能扩展
4. 已有的 `settings` 表设计（key-value）可扩展
5. 安全性更好（敏感配置不会暴露在前端）

| 设置类型 | 存储位置 | 理由 |
|---------|---------|------|
| 用户偏好（主题、语言） | SQLite | 持久化，后端可读 |
| API 密钥 | SQLite（加密存储） | 安全 |
| 快捷键配置 | SQLite | 跨设备同步 |
| UI 状态（临时） | SQLite | 持久化，后端可读 |

---

## 页面方案

**全屏路由页面**，原因：
1. 设置项众多（7个分类），抽屉空间有限
2. 支持更清晰的导航和面包屑
3. 便于后续添加子页面层级
4. 符合桌面应用设置页面的用户习惯

### 路由设计

```
/settings → SettingsPage (默认显示 general)
/settings/general → GeneralPanel
/settings/translation → TranslationPanel
/settings/learning → ReviewPanel
/settings/notification → NotificationPanel
/settings/appearance → AppearancePanel
/settings/shortcuts → ShortcutsPanel
/settings/about → AboutPanel
```

### 页面结构

```
SettingsPage
├── SettingsSidebar (左侧导航，固定宽度 w-60)
└── SettingsContent (右侧内容区，可滚动)
```

### 动画效果

**页面切换动画**:
- 左侧导航当前项高亮平滑过渡（background-color transition 200ms ease-out）
- 右侧内容区淡入：opacity 0→1, 200ms, ease-out
- 内容区轻微 slide-up 效果（translateY 8px→0）

**HomePage → Settings 页面切换**:
- 使用 React Router 的默认切换即可
- 可选增强：整个页面有 fade 效果

**示例 CSS**:
```css
.settings-content {
  animation: fadeSlideIn 200ms ease-out;
}
@keyframes fadeSlideIn {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

---

## 实施步骤

### Step 1: 创建 Settings Store

**文件**: `src/stores/settingsStore.ts`

```typescript
import { create } from 'zustand';

interface SettingsState {
  activeTab: string;
  settings: Record<string, string>;
  setActiveTab: (tab: string) => void;
  setSettings: (settings: Record<string, string>) => void;
  updateSetting: (key: string, value: string) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  activeTab: 'general',
  settings: {},
  setActiveTab: (activeTab) => set({ activeTab }),
  setSettings: (settings) => set({ settings }),
  updateSetting: (key, value) => set((state) => ({
    settings: { ...state.settings, [key]: value }
  })),
}));
```

### Step 2: 创建 Rust 后端设置命令

**文件**: `src-tauri/src/commands/settings.rs`（新建）

```rust
use std::collections::HashMap;

#[tauri::command]
pub fn get_settings(db: State<Database>) -> Result<HashMap<String, String>, String> {
    db.get_all_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(db: State<Database>, key: String, value: String) -> Result<(), String> {
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}
```

**更新**: `src-tauri/src/db/mod.rs` 添加 `get_all_settings`, `set_setting` 方法

### Step 3: 创建前端 API 封装

**文件**: `src/lib/api.ts`（添加）

```typescript
export async function getSettings(): Promise<Record<string, string>> {
  return invoke('get_settings');
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke('set_setting', { key, value });
}
```

### Step 4: 创建 SettingsPage 组件

**文件**: `src/pages/Settings/SettingsPage.tsx`

```tsx
import { useSettingsStore } from '../../stores/settingsStore';
import { SettingsSidebar } from './SettingsSidebar';
import { GeneralPanel } from './panels/GeneralPanel';
import { TranslationPanel } from './panels/TranslationPanel';
import { ReviewPanel } from './panels/ReviewPanel';
import { NotificationPanel } from './panels/NotificationPanel';
import { AppearancePanel } from './panels/AppearancePanel';
import { ShortcutsPanel } from './panels/ShortcutsPanel';
import { AboutPanel } from './panels/AboutPanel';

const panelComponents = {
  general: GeneralPanel,
  translation: TranslationPanel,
  learning: ReviewPanel,
  notification: NotificationPanel,
  appearance: AppearancePanel,
  shortcuts: ShortcutsPanel,
  about: AboutPanel,
};

export function SettingsPage() {
  const activeTab = useSettingsStore((state) => state.activeTab);
  const ActivePanel = panelComponents[activeTab as keyof typeof panelComponents];

  return (
    <div className="flex h-screen">
      <SettingsSidebar />
      <main className="flex-1 flex flex-col settings-content">
        <ActivePanel />
      </main>
    </div>
  );
}
```

### Step 5: 创建 SettingsSidebar 组件

**文件**: `src/pages/Settings/SettingsSidebar.tsx`

```tsx
import { ListBox, ListBoxItem } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';
import { Settings, Globe, Languages, BookOpen, Bell, Palette, Keyboard, Info } from 'lucide-react';

const navItems = [
  { key: 'general', label: '通用', icon: Settings },
  { key: 'translation', label: '翻译', icon: Languages },
  { key: 'learning', label: '学习', icon: BookOpen },
  { key: 'notification', label: '通知', icon: Bell },
  { key: 'appearance', label: '外观', icon: Palette },
  { key: 'shortcuts', label: '快捷键', icon: Keyboard },
  { key: 'about', label: '关于', icon: Info },
];

export function SettingsSidebar() {
  const { activeTab, setActiveTab } = useSettingsStore();

  return (
    <aside className="w-60 border-r border-divider bg-background p-4">
      <ListBox
        selectionMode="single"
        selectedKeys={[activeTab]}
        onSelectionChange={(keys) => {
          const key = Array.from(keys)[0] as string;
          if (key) setActiveTab(key);
        }}
      >
        {navItems.map((item) => (
          <ListBoxItem
            key={item.key}
            textValue={item.label}
            className="hover:text-[#3a6b49]"
          >
            <div className="flex items-center gap-3">
              <item.icon size={18} />
              <span>{item.label}</span>
            </div>
          </ListBoxItem>
        ))}
      </ListBox>
    </aside>
  );
}
```

### Step 6: 创建各设置面板组件

```
src/pages/Settings/panels/
├── GeneralPanel.tsx      # 通用设置
├── TranslationPanel.tsx   # 翻译设置
├── ReviewPanel.tsx       # 学习设置
├── NotificationPanel.tsx # 通知设置
├── AppearancePanel.tsx    # 外观设置
├── ShortcutsPanel.tsx     # 快捷键
└── AboutPanel.tsx        # 关于
```

**面板基础结构示例**:

```tsx
import { useSettingsStore } from '../../../stores/settingsStore';

export function GeneralPanel() {
  const { settings, updateSetting } = useSettingsStore();

  return (
    <div className="p-6 flex flex-col gap-6 settings-content">
      <h1 className="text-xl font-medium">通用设置</h1>

      {/* 系统主题 */}
      <div className="flex items-center justify-between">
        <span>系统主题</span>
        <input
          type="color"
          value={settings.theme || '#3a6b49'}
          onChange={(e) => updateSetting('theme', e.target.value)}
        />
      </div>

      {/* 语言 */}
      <div className="flex items-center justify-between">
        <span>语言</span>
        <select
          value={settings.language || 'zh'}
          onChange={(e) => updateSetting('language', e.target.value)}
        >
          <option value="zh">中文</option>
          <option value="en">English</option>
          <option value="system">跟随系统</option>
        </select>
      </div>
      {/* ... 更多设置项 */}
    </div>
  );
}
```

### Step 7: 添加路由配置

**文件**: `src/App.tsx`（更新）

```tsx
import { SettingsPage } from './pages/Settings/SettingsPage';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Routes>
    </BrowserRouter>
  );
}
```

### Step 8: 修改 HomePage 设置按钮

**文件**: `src/pages/Home/HomePage.tsx`

将按钮改为链接导航：

```tsx
import { Button } from '@heroui/react';
import { Settings } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

const navigate = useNavigate();

// 在 aside 底部
<div className="mt-auto pt-4 border-t border-divider">
  <Button
    variant="ghost"
    className="w-full justify-start"
    startContent={<Settings size={18} />}
    onPress={() => navigate('/settings')}
  >
    设置
  </Button>
</div>
```

---

## 关键文件

| 文件 | 操作 | 用途 |
|------|------|------|
| `src/stores/settingsStore.ts` | 创建 | Zustand 设置状态管理 |
| `src-tauri/src/commands/settings.rs` | 创建 | Rust 设置命令 |
| `src-tauri/src/db/mod.rs` | 修改 | 添加 settings 表操作方法 |
| `src/lib/api.ts` | 修改 | 添加 getSettings, setSetting 调用 |
| `src/pages/Settings/SettingsPage.tsx` | 创建 | 设置页面容器 |
| `src/pages/Settings/SettingsSidebar.tsx` | 创建 | 左侧导航 |
| `src/pages/Settings/panels/*.tsx` | 创建 | 各设置面板（7个） |
| `src/App.tsx` | 修改 | 添加路由配置 |
| `src/pages/Home/HomePage.tsx` | 修改 | 按钮改为导航 |

---

## 验证方式

1. `npm run tauri dev` 启动应用
2. 点击侧边栏底部"设置"按钮
3. 验证：
   - [ ] 页面正常切换到 /settings
   - [ ] 左侧导航高亮当前项
   - [ ] 右侧内容区有淡入动画（200ms）
   - [ ] 点击不同导航项，内容区平滑切换
   - [ ] 修改设置后刷新页面，值已保存到 SQLite
4. `npm run build` 构建成功