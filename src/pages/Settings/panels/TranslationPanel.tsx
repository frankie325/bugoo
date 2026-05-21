import { useSettingsStore } from "../../../stores/settingsStore";
import {
  Card,
  Select,
  ListBox,
  Input,
  Separator,
  Label,
  TextArea,
  NumberField,
} from "@heroui/react";
import { SettingItem } from "../components/SettingItem";
import { useTranslation } from "react-i18next";
import { setSetting } from "../../../lib/api";

const engineOptionGroups = [
  {
    i18nKey: "engineGroupSystem",
    options: [{ i18nKey: "engineLibreTranslate", value: "libretranslate" }],
  },
  {
    i18nKey: "engineGroupVendor",
    options: [
      { i18nKey: "engineGoogle", value: "google" },
      { i18nKey: "engineDeepL", value: "deepl" },
      { i18nKey: "engineMicrosoft", value: "microsoft" },
      { i18nKey: "engineBaidu", value: "baidu" },
      { i18nKey: "engineTencent", value: "tencent" },
      { i18nKey: "engineYoudao", value: "youdao" },
    ],
  },
  {
    i18nKey: "engineGroupCustom",
    options: [{ i18nKey: "engineCustom", value: "custom" }],
  },
];

const DEFAULT_TRANSLATION_TIMEOUT_MS = 15000;

export function TranslationPanel() {
  const { t } = useTranslation();
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);

  const saveSetting = (key: string, value: string) => {
    updateSetting(key, value);
    setSetting(key, value).catch((error) => {
      console.error(`保存设置失败：${key}`, error);
    });
  };

  const rawEngine = settings.translationEngine || "libretranslate";
  const translationEngine = rawEngine === "openai" ? "custom" : rawEngine;
  const apiEndpoint = settings.apiEndpoint || "";
  const apiKey = settings.apiKey || "";
  const apiSecret = settings.apiSecret || "";
  const apiRegion = settings.apiRegion || "";
  const translationModel = settings.translationModel || "";
  const translationPrompt = settings.translationPrompt || "";
  const wordDetailPrompt = settings.wordDetailPrompt || "";
  const parsedTranslationTimeoutMs = Number(
    settings.translationTimeoutMs || String(DEFAULT_TRANSLATION_TIMEOUT_MS),
  );
  const translationTimeoutMs = Number.isFinite(parsedTranslationTimeoutMs)
    ? parsedTranslationTimeoutMs
    : DEFAULT_TRANSLATION_TIMEOUT_MS;
  const needsEndpoint = [
    "libretranslate",
    "google",
    "deepl",
    "microsoft",
    "custom",
  ].includes(translationEngine);
  const endpointOptional = ["google", "deepl", "microsoft"].includes(
    translationEngine,
  );
  const needsApiKey = [
    "libretranslate",
    "google",
    "deepl",
    "microsoft",
    "baidu",
    "tencent",
    "youdao",
    "custom",
  ].includes(translationEngine);
  const apiKeyOptional = translationEngine === "libretranslate";
  const needsApiSecret = ["baidu", "tencent", "youdao"].includes(
    translationEngine,
  );
  const needsApiRegion = ["microsoft", "tencent"].includes(translationEngine);
  const isCustomEngine = translationEngine === "custom";

  // Migrate "openai" → "custom" for existing users
  if (rawEngine === "openai") {
    saveSetting("translationEngine", "custom");
  }

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
              value && saveSetting("translationEngine", String(value))
            }
          >
            <Label>{t("settings.translation.engineLabel")}</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {engineOptionGroups.flatMap((group) => [
                  <ListBox.Item
                    key={group.i18nKey}
                    id={group.i18nKey}
                    textValue={t(`settings.translation.${group.i18nKey}`)}
                    isDisabled
                  >
                    <span className="text-xs font-medium text-default-500">
                      {t(`settings.translation.${group.i18nKey}`)}
                    </span>
                  </ListBox.Item>,
                  ...group.options.map((opt) => (
                    <ListBox.Item
                      key={opt.value}
                      id={opt.value}
                      textValue={t(`settings.translation.${opt.i18nKey}`)}
                    >
                      {t(`settings.translation.${opt.i18nKey}`)}
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  )),
                ])}
              </ListBox>
            </Select.Popover>
          </Select>
        </SettingItem>
        {needsEndpoint && (
          <>
            <Separator />
            <SettingItem
              title={t("settings.translation.endpoint.title")}
              description={
                endpointOptional
                  ? t("settings.translation.endpoint.optionalDesc")
                  : t("settings.translation.endpoint.desc")
              }
            >
              <Input
                value={apiEndpoint}
                onChange={(e) => saveSetting("apiEndpoint", e.target.value)}
                placeholder={t(
                  `settings.translation.endpointPlaceholder.${translationEngine}`,
                  {
                    defaultValue: t(
                      "settings.translation.endpointPlaceholder.default",
                    ),
                  },
                )}
                className="w-64"
              />
            </SettingItem>
          </>
        )}

        {needsApiKey && (
          <>
            <Separator />
            <SettingItem
              title={t("settings.translation.apiKey.title")}
              description={
                apiKeyOptional
                  ? t("settings.translation.apiKey.optionalDesc")
                  : t("settings.translation.apiKey.desc")
              }
            >
              <Input
                value={apiKey}
                onChange={(e) => saveSetting("apiKey", e.target.value)}
                placeholder={t("settings.translation.apiKeyPlaceholder")}
                type="password"
                className="w-64"
              />
            </SettingItem>
          </>
        )}

        {needsApiSecret && (
          <>
            <Separator />
            <SettingItem
              title={t("settings.translation.apiSecret.title")}
              description={t("settings.translation.apiSecret.desc")}
            >
              <Input
                value={apiSecret}
                onChange={(e) => saveSetting("apiSecret", e.target.value)}
                placeholder={t("settings.translation.apiSecretPlaceholder")}
                type="password"
                className="w-64"
              />
            </SettingItem>
          </>
        )}

        {needsApiRegion && (
          <>
            <Separator />
            <SettingItem
              title={t("settings.translation.apiRegion.title")}
              description={t("settings.translation.apiRegion.desc")}
            >
              <Input
                value={apiRegion}
                onChange={(e) => saveSetting("apiRegion", e.target.value)}
                placeholder={t("settings.translation.apiRegionPlaceholder")}
                className="w-64"
              />
            </SettingItem>
          </>
        )}

        {isCustomEngine && (
          <>
            <Separator />
            <SettingItem
              title={t("settings.translation.model.title")}
              description={t("settings.translation.model.desc")}
            >
              <Input
                value={translationModel}
                onChange={(e) => saveSetting("translationModel", e.target.value)}
                placeholder={t("settings.translation.modelPlaceholder")}
                className="w-64"
              />
            </SettingItem>

            <Separator />
            <SettingItem
              title={t("settings.translation.prompt.title")}
              description={t("settings.translation.prompt.desc")}
            >
              <TextArea
                value={translationPrompt}
                onChange={(e) => saveSetting("translationPrompt", e.target.value)}
                placeholder={t("settings.translation.promptPlaceholder")}
                className="w-80"
              />
            </SettingItem>

            <Separator />
            <SettingItem
              title={t("settings.translation.wordDetailPrompt.title")}
              description={t("settings.translation.wordDetailPrompt.desc")}
            >
              <TextArea
                value={wordDetailPrompt}
                onChange={(e) => saveSetting("wordDetailPrompt", e.target.value)}
                placeholder={t("settings.translation.wordDetailPromptPlaceholder")}
                className="w-80"
              />
            </SettingItem>
          </>
        )}

        <Separator />

        {/* 超时时间 */}
        <SettingItem title={t("settings.translation.timeout.title")} description={t("settings.translation.timeout.desc")}>
          <div className="flex items-center gap-2">
            <NumberField
              minValue={1000}
              value={translationTimeoutMs}
              onChange={(value) =>
                saveSetting(
                  "translationTimeoutMs",
                  String(
                    Number.isFinite(value)
                      ? value
                      : DEFAULT_TRANSLATION_TIMEOUT_MS,
                  ),
                )
              }
              className="w-36"
            >
              <NumberField.Group>
                <NumberField.DecrementButton />
                <NumberField.Input />
                <NumberField.IncrementButton />
              </NumberField.Group>
            </NumberField>
            <Label>{t("settings.translation.timeoutLabel")}</Label>
          </div>
        </SettingItem>
      </Card.Content>
    </Card>
  );
}
