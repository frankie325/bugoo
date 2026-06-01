import { useEffect, useState, useRef } from "react";
import { useSettingsStore } from "../../../stores/settingsStore";
import {
  Card,
  Select,
  ListBox,
  Input,
  InputGroup,
  Separator,
  Label,
  TextArea,
  NumberField,
  Button,
} from "@heroui/react";
import { SettingItem } from "../components/SettingItem";
import { useTranslation } from "react-i18next";
import { setSetting, getTranslationLanguages } from "../../../lib/api";
import {
  getTranslationCredentialFieldHints,
  getTranslationFieldVisibility,
  getFilteredTargetLanguages,
  emptyTranslationLanguages,
  hasLanguage,
  localizeTranslationLanguageName,
  type TranslationEngine,
  type TranslationLanguages,
  type TranslationLanguage,
} from "./translationSettingsModel";
import { Eye, EyeOff } from "lucide-react";

const engineOptionGroups = [
  {
    i18nKey: "engineGroupLocal",
    options: [{ i18nKey: "engineLocal", value: "local" }],
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

function CredentialFieldTitle({
  title,
  hint,
}: {
  title: string;
  hint?: string;
}) {
  if (!hint) {
    return title;
  }

  return (
    <span>
      {title}
      <span className="ml-1 text-xs font-normal text-default-500">
        ({hint})
      </span>
    </span>
  );
}

export function TranslationPanel() {
  const { t, i18n } = useTranslation();
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);
  const [translationLanguages, setTranslationLanguages] =
    useState<TranslationLanguages>(emptyTranslationLanguages);
  const [isSecretVisible, setIsSecretVisible] = useState(false);
  const [isApiKeyVisible, setIsApiKeyVisible] = useState(false);
  const prevEngineRef = useRef<string | undefined>(undefined);

  useEffect(() => {
    let disposed = false;
    const engine = settings.translationEngine || "local";
    const prevEngine = prevEngineRef.current;

    getTranslationLanguages(engine)
      .then((languages) => {
        if (!disposed) {
          setTranslationLanguages(languages);
          // Only reset language selection when engine actually changes
          if (prevEngine !== undefined && prevEngine !== engine) {
            if (languages.sourceLanguages.length > 0) {
              const firstSource = languages.sourceLanguages[0].code;
              saveSetting("sourceLanguage", firstSource);

              // 根据 sourceToTargetMapping 获取目标语言
              const allowedTargets = languages.sourceToTargetMapping?.[firstSource];
              if (allowedTargets && allowedTargets.length > 0) {
                saveSetting("targetLanguage", allowedTargets[0]);
              } else if (languages.targetLanguages.length > 0) {
                saveSetting("targetLanguage", languages.targetLanguages[0].code);
              }
            }
          }
          prevEngineRef.current = engine;
        }
      })
      .catch((error) => {
        console.error("加载翻译语言失败", error);
      });

    return () => {
      disposed = true;
    };
  }, [settings.translationEngine]);

  const saveSetting = (key: string, value: string) => {
    updateSetting(key, value);
    setSetting(key, value).catch((error) => {
      console.error(`保存设置失败：${key}`, error);
    });
  };

  const translationEngine = (settings.translationEngine ||
    "local") as TranslationEngine;
  const sourceLanguage = hasLanguage(
    translationLanguages.sourceLanguages,
    settings.sourceLanguage || "auto",
  )
    ? settings.sourceLanguage || "auto"
    : "auto";
  const targetLanguage = hasLanguage(
    translationLanguages.targetLanguages,
    settings.targetLanguage || "zh",
  )
    ? settings.targetLanguage || "zh"
    : "zh";
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
  const fieldVisibility = getTranslationFieldVisibility(translationEngine);
  const credentialFieldHints =
    getTranslationCredentialFieldHints(translationEngine);
  const filteredTargetLanguages = getFilteredTargetLanguages(
    sourceLanguage,
    translationLanguages,
  );
  const locale = i18n.resolvedLanguage || i18n.language;
  const getLanguageName = (language: TranslationLanguage) =>
    localizeTranslationLanguageName(language, locale, {
      autoDetectName: t("settings.translation.autoDetectLanguage"),
    });

  return (
    <Card>
      <Card.Header>
        <Card.Title>{t("settings.translation.title")}</Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 翻译引擎 */}
        <SettingItem
          title={t("settings.translation.engine.title")}
          description={t("settings.translation.engine.desc")}
        >
          <Select
            className="w-48"
            value={translationEngine}
            onChange={(value) =>
              value && saveSetting("translationEngine", String(value))
            }
          >
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

        <Separator />
        <SettingItem
          title={t("settings.translation.sourceLanguage.title")}
          description={t("settings.translation.sourceLanguage.desc")}
        >
          <Select
            className="w-48"
            value={sourceLanguage}
            onChange={(value) => {
              if (value) {
                saveSetting("sourceLanguage", String(value));
                const filtered = getFilteredTargetLanguages(
                  String(value),
                  translationLanguages,
                );
                if (filtered.length > 0) {
                  saveSetting("targetLanguage", filtered[0].code);
                }
              }
            }}
          >
            <Label>{t("settings.translation.sourceLanguage.label")}</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {translationLanguages.sourceLanguages.map((option) => (
                  <ListBox.Item
                    key={option.code}
                    id={option.code}
                    textValue={getLanguageName(option)}
                  >
                    {getLanguageName(option)}
                    <ListBox.ItemIndicator />
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>
        </SettingItem>

        <Separator />
        <SettingItem
          title={t("settings.translation.targetLanguage.title")}
          description={t("settings.translation.targetLanguage.desc")}
        >
          <Select
            className="w-48"
            value={targetLanguage}
            onChange={(value) => {
              if (value) {
                const strValue = String(value);
                const allowed = translationLanguages.sourceToTargetMapping
                  ? translationLanguages.sourceToTargetMapping[
                      sourceLanguage
                    ]
                  : null;
                if (
                  allowed &&
                  allowed.length > 0 &&
                  !allowed.includes(strValue)
                ) {
                  return;
                }
                saveSetting("targetLanguage", strValue);
              }
            }}
          >
            <Label>{t("settings.translation.targetLanguage.label")}</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {filteredTargetLanguages.map((option) => (
                  <ListBox.Item
                    key={option.code}
                    id={option.code}
                    textValue={getLanguageName(option)}
                  >
                    {getLanguageName(option)}
                    <ListBox.ItemIndicator />
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>
        </SettingItem>

        {fieldVisibility.needsEndpoint && (
          <>
            <Separator />
            <SettingItem
              title={t("settings.translation.endpoint.title")}
              description={
                fieldVisibility.endpointOptional
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

        {fieldVisibility.needsApiKey && (
          <>
            <Separator />
            <SettingItem
              title={
                <CredentialFieldTitle
                  title={t("settings.translation.apiKey.title")}
                  hint={credentialFieldHints.apiKey}
                />
              }
              description={
                fieldVisibility.endpointOptional
                  ? t("settings.translation.apiKey.optionalDesc")
                  : t("settings.translation.apiKey.desc")
              }
            >
              <InputGroup>
                <InputGroup.Input
                  value={apiKey}
                  onChange={(e) => saveSetting("apiKey", e.target.value)}
                  placeholder={t("settings.translation.apiKeyPlaceholder")}
                  type={isApiKeyVisible ? "text" : "password"}
                  className="w-64"
                />
                <InputGroup.Suffix className="pr-0">
                  <Button
                    isIconOnly
                    size="sm"
                    variant="ghost"
                    onPress={() => setIsApiKeyVisible(!isApiKeyVisible)}
                  >
                    {isApiKeyVisible ? <EyeOff size={18} /> : <Eye size={18} />}
                  </Button>
                </InputGroup.Suffix>
              </InputGroup>
            </SettingItem>
          </>
        )}

        {fieldVisibility.needsApiSecret && (
          <>
            <Separator />
            <SettingItem
              title={
                <CredentialFieldTitle
                  title={t("settings.translation.apiSecret.title")}
                  hint={credentialFieldHints.apiSecret}
                />
              }
              description={t("settings.translation.apiSecret.desc")}
            >
              <InputGroup>
                <InputGroup.Input
                  value={apiSecret}
                  onChange={(e) => saveSetting("apiSecret", e.target.value)}
                  placeholder={t("settings.translation.apiSecretPlaceholder")}
                  type={isSecretVisible ? "text" : "password"}
                  className="w-64"
                />
                <InputGroup.Suffix className="pr-0">
                  <Button
                    isIconOnly
                    size="sm"
                    variant="ghost"
                    onPress={() => setIsSecretVisible(!isSecretVisible)}
                  >
                    {isSecretVisible ? <EyeOff size={18} /> : <Eye size={18} />}
                  </Button>
                </InputGroup.Suffix>
              </InputGroup>
            </SettingItem>
          </>
        )}

        {fieldVisibility.needsApiRegion && (
          <>
            <Separator />
            <SettingItem
              title={
                <CredentialFieldTitle
                  title={t("settings.translation.apiRegion.title")}
                  hint={credentialFieldHints.apiRegion}
                />
              }
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

        {fieldVisibility.isCustomEngine && (
          <>
            <Separator />
            <SettingItem
              title={t("settings.translation.model.title")}
              description={t("settings.translation.model.desc")}
            >
              <Input
                value={translationModel}
                onChange={(e) =>
                  saveSetting("translationModel", e.target.value)
                }
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
                onChange={(e) =>
                  saveSetting("translationPrompt", e.target.value)
                }
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
                onChange={(e) =>
                  saveSetting("wordDetailPrompt", e.target.value)
                }
                placeholder={t(
                  "settings.translation.wordDetailPromptPlaceholder",
                )}
                className="w-80"
              />
            </SettingItem>
          </>
        )}

        <Separator />

        {/* 超时时间 */}
        <SettingItem
          title={t("settings.translation.timeout.title")}
          description={t("settings.translation.timeout.desc")}
        >
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
