import { useSettingsStore } from "../../../stores/settingsStore";
import { Card, Kbd, Separator } from "@heroui/react";
import { SettingItem } from "../components/SettingItem";
import { useTranslation } from "react-i18next";

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
  const { t } = useTranslation();
  const settings = useSettingsStore((state) => state.settings);

  const shortcutStartReview = settings.shortcutStartReview || "Cmd+Enter";
  const shortcutTranslation = settings.shortcutTranslation || "Cmd+Shift+B";
  const shortcutNewWord = settings.shortcutNewWord || "Cmd+D";
  const shortcutOpenApp = settings.shortcutOpenApp || "Cmd+K";

  return (
    <Card>
      <Card.Header>
        <Card.Title>{t("settings.shortcuts.title")}</Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 开始复习 */}
        <SettingItem title={t("settings.shortcuts.startReview.title")} description={t("settings.shortcuts.startReview.desc")}>
          <ShortcutDisplay shortcut={shortcutStartReview} />
        </SettingItem>

        <Separator />

        {/* 划词翻译 */}
        <SettingItem title={t("settings.shortcuts.translation.title")} description={t("settings.shortcuts.translation.desc")}>
          <ShortcutDisplay shortcut={shortcutTranslation} />
        </SettingItem>

        <Separator />

        {/* 快速添加单词 */}
        <SettingItem title={t("settings.shortcuts.addWord.title")} description={t("settings.shortcuts.addWord.desc")}>
          <ShortcutDisplay shortcut={shortcutNewWord} />
        </SettingItem>

        <Separator />

        {/* 打开主界面 */}
        <SettingItem title={t("settings.shortcuts.openApp.title")} description={t("settings.shortcuts.openApp.desc")}>
          <ShortcutDisplay shortcut={shortcutOpenApp} />
        </SettingItem>
      </Card.Content>
    </Card>
  );
}
