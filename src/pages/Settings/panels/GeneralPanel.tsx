import { useSettingsStore } from "../../../stores/settingsStore";
import {
  Card,
  Switch,
  Select,
  ListBox,
  RadioGroup,
  Radio,
  Separator,
  ColorSwatchPicker,
  Text,
} from "@heroui/react";
import { SettingItem } from "../components/SettingItem";
import i18n from "../../../lib/i18n";
import { useTranslation } from "react-i18next";

const languageLabels: Record<string, string> = {
  "zh-CN": "简体中文",
  "zh-TW": "繁體中文",
  en: "English",
  ja: "日本語",
  ko: "한국어",
  es: "Español",
  fr: "Français",
  de: "Deutsch",
  pt: "Português",
  ru: "Русский",
  ar: "العربية",
  hi: "हिन्दी",
  th: "ไทย",
  vi: "Tiếng Việt",
  id: "Bahasa Indonesia",
};

const languageOptions = Object.keys(i18n.options.resources || {}).map((code) => ({
  label: languageLabels[code] || code,
  value: code,
}));

const themeColorKeys = [
  { colorKey: "green", color: "#10b981" },
  { colorKey: "blue", color: "#3b82f6" },
  { colorKey: "purple", color: "#8b5cf6" },
];

export function GeneralPanel() {
  const { t } = useTranslation();
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);

  const language = settings.language || "zh-CN";
  const startup = settings.startup === "true";
  const closeBehavior = settings.closeBehavior || "minimize";
  const autoUpdate = settings.autoUpdate !== "false";
  const theme = settings.theme || "#10b981";

  const handleLanguageChange = (newLang: string) => {
    updateSetting("language", newLang);
    i18n.changeLanguage(newLang);
  };

  return (
    <Card>
      <Card.Header>
        <Card.Title>
          <Text type="h3">{t("settings.general.title")}</Text>
        </Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 主题色 */}
        <SettingItem title={t("settings.general.themeColor.title")} description={t("settings.general.themeColor.desc")}>
          <ColorSwatchPicker
            value={theme}
            onChange={(color) =>
              updateSetting("theme", color as unknown as string)
            }
          >
            {themeColorKeys.map((c) => (
              <ColorSwatchPicker.Item key={c.colorKey} color={c.color}>
                <ColorSwatchPicker.Swatch color={c.color} />
              </ColorSwatchPicker.Item>
            ))}
          </ColorSwatchPicker>
        </SettingItem>

        <Separator />

        {/* 启动时打开 */}
        <SettingItem title={t("settings.general.startup.title")} description={t("settings.general.startup.desc")}>
          <Switch
            isSelected={startup}
            onChange={(val) => updateSetting("startup", String(val))}
          >
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
          </Switch>
        </SettingItem>

        <Separator />

        {/* 自动更新 */}
        <SettingItem title={t("settings.general.autoUpdate.title")} description={t("settings.general.autoUpdate.desc")}>
          <Switch
            isSelected={autoUpdate}
            onChange={(val) => updateSetting("autoUpdate", String(val))}
          >
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
          </Switch>
        </SettingItem>

        <Separator />

        {/* 关闭行为 */}
        <SettingItem title={t("settings.general.closeBehavior.title")} description={t("settings.general.closeBehavior.desc")}>
          <RadioGroup
            orientation="horizontal"
            value={closeBehavior}
            onChange={(val) => updateSetting("closeBehavior", val)}
          >
            <Radio value="minimize">
              <Radio.Control>
                <Radio.Indicator />
              </Radio.Control>
              {t("settings.general.minimize")}
            </Radio>
            <Radio value="quit">
              <Radio.Control>
                <Radio.Indicator />
              </Radio.Control>
              {t("settings.general.quit")}
            </Radio>
          </RadioGroup>
        </SettingItem>

        <Separator />

        {/* 界面语言 */}
        <SettingItem title={t("settings.general.language.title")} description={t("settings.general.language.desc")}>
          <Select
            className="w-36"
            value={language}
            onChange={(value) => value && handleLanguageChange(String(value))}
          >
            <Select.Trigger>
              <Select.Value />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {languageOptions.map((l) => (
                  <ListBox.Item key={l.value} id={l.value} textValue={l.label}>
                    {l.label}
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>
        </SettingItem>
      </Card.Content>
    </Card>
  );
}
