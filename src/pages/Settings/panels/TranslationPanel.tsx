import { useSettingsStore } from "../../../stores/settingsStore";
import { Card, Select, ListBox, Input, Separator, Label } from "@heroui/react";
import { SettingItem } from "../components/SettingItem";
import { useTranslation } from "react-i18next";

const engineOptionKeys = [
  { i18nKey: "engineDeepL", value: "deepl" },
  { i18nKey: "engineGoogle", value: "google" },
  { i18nKey: "engineOpenAI", value: "openai" },
  { i18nKey: "engineCustom", value: "custom" },
];

export function TranslationPanel() {
  const { t } = useTranslation();
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);

  const translationEngine = settings.translationEngine || "deepl";
  const apiEndpoint = settings.apiEndpoint || "";
  const apiKey = settings.apiKey || "";

  return (
    <Card>
      <Card.Header>
        <Card.Title>{t("settings.translation.title")}</Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 翻译引擎 */}
        <SettingItem title={t("settings.translation.engine.title")} description={t("settings.translation.engine.desc")}>
          <Select
            className="w-40"
            value={translationEngine}
            onChange={(value) =>
              value && updateSetting("translationEngine", String(value))
            }
          >
            <Label>{t("settings.translation.engineLabel")}</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {engineOptionKeys.map((opt) => (
                  <ListBox.Item
                    key={opt.value}
                    id={opt.value}
                    textValue={t(`settings.translation.${opt.i18nKey}`)}
                  >
                    {t(`settings.translation.${opt.i18nKey}`)}
                    <ListBox.ItemIndicator />
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>
        </SettingItem>

        <Separator />

        {/* API地址 */}
        <SettingItem title={t("settings.translation.endpoint.title")} description={t("settings.translation.endpoint.desc")}>
          <Input
            value={apiEndpoint}
            onChange={(e) => updateSetting("apiEndpoint", e.target.value)}
            placeholder={t("settings.translation.endpointPlaceholder")}
            className="w-64"
          />
        </SettingItem>

        <Separator />

        {/* API秘钥 */}
        <SettingItem title={t("settings.translation.apiKey.title")} description={t("settings.translation.apiKey.desc")}>
          <Input
            value={apiKey}
            onChange={(e) => updateSetting("apiKey", e.target.value)}
            placeholder={t("settings.translation.apiKeyPlaceholder")}
            type="password"
            className="w-64"
          />
        </SettingItem>
      </Card.Content>
    </Card>
  );
}
