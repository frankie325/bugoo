import { useSettingsStore } from "../../../stores/settingsStore";
import {
  Card,
  Tabs,
  RadioGroup,
  Radio,
  Separator,
  ColorSwatchPicker,
  parseColor,
} from "@heroui/react";
import { SettingItem } from "../components/SettingItem";
import { useTranslation } from "react-i18next";

const themeColorKeys = [
  { colorKey: "green", color: "#10b981" },
  { colorKey: "blue", color: "#3b82f6" },
  { colorKey: "purple", color: "#8b5cf6" },
];

const cardStyleKeys = [
  { i18nKey: "styleSimple", value: "simple" },
  { i18nKey: "styleRich", value: "rich" },
];

const fontSizeKeys = [
  { i18nKey: "fontSmall", value: "small" },
  { i18nKey: "fontMedium", value: "medium" },
  { i18nKey: "fontLarge", value: "large" },
];

export function AppearancePanel() {
  const { t } = useTranslation();
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);

  const theme = settings.theme || "light";
  const themeColor = settings.themeColor || "#10b981";
  const cardStyle = settings.cardStyle || "rich";
  const fontSize = settings.fontSize || "medium";

  const colorValue = parseColor(themeColor);

  return (
    <Card>
      <Card.Header>
        <Card.Title>{t("settings.appearance.title")}</Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 主题 */}
        <SettingItem title={t("settings.appearance.theme.title")} description={t("settings.appearance.theme.desc")}>
          <Tabs
            selectedKey={theme}
            onSelectionChange={(key) => updateSetting("theme", key as string)}
          >
            <Tabs.ListContainer>
              <Tabs.List aria-label={t("settings.appearance.themeAriaLabel")}>
                <Tabs.Tab id="light">
                  {t("settings.appearance.themeLight")}
                  <Tabs.Indicator />
                </Tabs.Tab>
                <Tabs.Tab id="dark">
                  {t("settings.appearance.themeDark")}
                  <Tabs.Indicator />
                </Tabs.Tab>
                <Tabs.Tab id="system">
                  {t("settings.appearance.themeSystem")}
                  <Tabs.Indicator />
                </Tabs.Tab>
              </Tabs.List>
            </Tabs.ListContainer>
          </Tabs>
        </SettingItem>

        <Separator />

        {/* 主题颜色 */}
        <SettingItem title={t("settings.appearance.themeColor.title")} description={t("settings.appearance.themeColor.desc")}>
          <ColorSwatchPicker
            value={colorValue}
            onChange={(color) =>
              updateSetting("themeColor", color.toString("hex"))
            }
          >
            {themeColorKeys.map((c) => (
              <ColorSwatchPicker.Item key={c.colorKey} color={c.color}>
                <ColorSwatchPicker.Swatch color={c.color} />
                <ColorSwatchPicker.Indicator />
              </ColorSwatchPicker.Item>
            ))}
          </ColorSwatchPicker>
        </SettingItem>

        <Separator />

        {/* 卡片风格 */}
        <SettingItem title={t("settings.appearance.cardStyle.title")} description={t("settings.appearance.cardStyle.desc")}>
          <RadioGroup
            className="flex flex-row gap-4"
            value={cardStyle}
            onChange={(val) => updateSetting("cardStyle", val)}
          >
            {cardStyleKeys.map((opt) => (
              <Radio key={opt.value} value={opt.value}>
                <Radio.Control>
                  <Radio.Indicator />
                </Radio.Control>
                {t(`settings.appearance.${opt.i18nKey}`)}
              </Radio>
            ))}
          </RadioGroup>
        </SettingItem>

        <Separator />

        {/* 字体大小 */}
        <SettingItem title={t("settings.appearance.fontSize.title")} description={t("settings.appearance.fontSize.desc")}>
          <Tabs
            selectedKey={fontSize}
            onSelectionChange={(key) =>
              updateSetting("fontSize", key as string)
            }
          >
            <Tabs.ListContainer>
              <Tabs.List aria-label={t("settings.appearance.fontSizeAriaLabel")}>
                {fontSizeKeys.map((opt) => (
                  <Tabs.Tab key={opt.value} id={opt.value}>
                    {t(`settings.appearance.${opt.i18nKey}`)}
                    <Tabs.Indicator />
                  </Tabs.Tab>
                ))}
              </Tabs.List>
            </Tabs.ListContainer>
          </Tabs>
        </SettingItem>
      </Card.Content>
    </Card>
  );
}
