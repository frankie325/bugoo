import { useSettingsStore } from "../../../stores/settingsStore";
import { Card, Kbd, Separator } from "@heroui/react";
import { SettingItem } from "../components/SettingItem";

function ShortcutDisplay({ shortcut }: { shortcut: string }) {
  const keys = shortcut.split("+");
  return (
    <div className="flex gap-1">
      {keys.map((key, i) => (
        <Kbd key={i}>{key}</Kbd>
      ))}
    </div>
  );
}

export function ShortcutsPanel() {
  const settings = useSettingsStore((state) => state.settings);

  const shortcutStartReview = settings.shortcutStartReview || "Cmd+Enter";
  const shortcutTranslation = settings.shortcutTranslation || "Cmd+Shift+B";
  const shortcutNewWord = settings.shortcutNewWord || "Cmd+D";
  const shortcutOpenApp = settings.shortcutOpenApp || "Cmd+K";

  return (
    <Card>
      <Card.Header>
        <Card.Title>快捷键</Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 开始复习 */}
        <SettingItem title="开始复习" description="快速开始一轮复习">
          <ShortcutDisplay shortcut={shortcutStartReview} />
        </SettingItem>

        <Separator />

        {/* 划词翻译 */}
        <SettingItem title="划词翻译" description="选中文本后自动翻译">
          <ShortcutDisplay shortcut={shortcutTranslation} />
        </SettingItem>

        <Separator />

        {/* 快速添加单词 */}
        <SettingItem title="快速添加单词" description="手动添加生词到词库">
          <ShortcutDisplay shortcut={shortcutNewWord} />
        </SettingItem>

        <Separator />

        {/* 打开主界面 */}
        <SettingItem title="打开主界面" description="显示主窗口">
          <ShortcutDisplay shortcut={shortcutOpenApp} />
        </SettingItem>
      </Card.Content>
    </Card>
  );
}
