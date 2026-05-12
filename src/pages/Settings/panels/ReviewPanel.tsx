import { useSettingsStore } from "../../../stores/settingsStore";
import {
  Card,
  Switch,
  RadioGroup,
  Radio,
  Separator,
  NumberField,
} from "@heroui/react";
import { SettingItem } from "../components/SettingItem";
import { useTranslation } from "react-i18next";

const dailyLimitOptions = [
  { label: "10", value: "10" },
  { label: "20", value: "20" },
  { label: "50", value: "50" },
];

const reviewPaceKeys = [
  { i18nKey: "relaxed", value: "relaxed" },
  { i18nKey: "normal", value: "normal" },
  { i18nKey: "intense", value: "intense" },
];

const hintStrategyKeys = [
  { i18nKey: "progressive", value: "progressive" },
  { i18nKey: "immediate", value: "immediate" },
];

export function ReviewPanel() {
  const { t } = useTranslation();
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);

  const dailyLimit = settings.dailyLimit || "20";
  const reviewPace = settings.reviewPace || "normal";
  const hintStrategy = settings.hintStrategy || "progressive";
  const enableSelection = settings.enableSelection !== "false";
  const autoSpeak = settings.autoSpeak === "true";
  const autoClose = settings.autoClose !== "false";

  const isCustomLimit = dailyLimitOptions.every(
    (opt) => opt.value !== dailyLimit,
  );

  return (
    <div className="space-y-6">
      {/* 复习设置 */}
      <Card>
        <Card.Header>
          <Card.Title>{t("settings.review.title")}</Card.Title>
        </Card.Header>
        <Card.Content>
          {/* 每日复习上限 */}
          <SettingItem title={t("settings.review.dailyLimit.title")} description={t("settings.review.dailyLimit.desc")}>
            <RadioGroup
              orientation="horizontal"
              value={isCustomLimit ? "custom" : dailyLimit}
              onChange={(val) => {
                if (val === "custom") {
                  updateSetting("dailyLimit", "20");
                } else {
                  updateSetting("dailyLimit", val);
                }
              }}
            >
              {dailyLimitOptions.map((opt) => (
                <Radio key={opt.value} value={opt.value}>
                  <Radio.Control>
                    <Radio.Indicator />
                  </Radio.Control>
                  {opt.label}
                </Radio>
              ))}
              <Radio value="custom">
                <Radio.Control>
                  <Radio.Indicator />
                </Radio.Control>
                {t("settings.review.custom")}
              </Radio>
            </RadioGroup>
          </SettingItem>

          {isCustomLimit && (
            <div className="flex justify-end mt-2">
              <NumberField
                minValue={1}
                maxValue={500}
                value={parseInt(dailyLimit) || 20}
                onChange={(val) =>
                  updateSetting("dailyLimit", String(val ?? 20))
                }
                className="w-24"
              />
            </div>
          )}

          <Separator />

          {/* 复习节奏 */}
          <SettingItem title={t("settings.review.reviewPace.title")} description={t("settings.review.reviewPace.desc")}>
            <RadioGroup
              orientation="horizontal"
              value={reviewPace}
              onChange={(val) => updateSetting("reviewPace", val)}
            >
              {reviewPaceKeys.map((opt) => (
                <Radio key={opt.value} value={opt.value}>
                  <Radio.Control>
                    <Radio.Indicator />
                  </Radio.Control>
                  {t(`settings.review.pace.${opt.i18nKey}`)}
                </Radio>
              ))}
            </RadioGroup>
          </SettingItem>

          <Separator />

          {/* 提示策略 */}
          <SettingItem title={t("settings.review.hintStrategy.title")} description={t("settings.review.hintStrategy.desc")}>
            <RadioGroup
              orientation="horizontal"
              value={hintStrategy}
              onChange={(val) => updateSetting("hintStrategy", val)}
            >
              {hintStrategyKeys.map((opt) => (
                <Radio key={opt.value} value={opt.value}>
                  <Radio.Control>
                    <Radio.Indicator />
                  </Radio.Control>
                  {t(`settings.review.hint.${opt.i18nKey}`)}
                </Radio>
              ))}
            </RadioGroup>
          </SettingItem>
        </Card.Content>
      </Card>

      {/* 划词设置 */}
      <Card>
        <Card.Header>
          <Card.Title>{t("settings.review.selection.title")}</Card.Title>
        </Card.Header>
        <Card.Content>
          {/* 是否启用划词翻译 */}
          <SettingItem
            title={t("settings.review.selectionEnable.title")}
            description={t("settings.review.selectionEnable.desc")}
          >
            <Switch
              isSelected={enableSelection}
              onChange={(val) => updateSetting("enableSelection", String(val))}
            >
              <Switch.Control>
                <Switch.Thumb />
              </Switch.Control>
            </Switch>
          </SettingItem>

          <Separator />

          {/* 自动发音 */}
          <SettingItem title={t("settings.review.autoSpeak.title")} description={t("settings.review.autoSpeak.desc")}>
            <Switch
              isSelected={autoSpeak}
              onChange={(val) => updateSetting("autoSpeak", String(val))}
            >
              <Switch.Control>
                <Switch.Thumb />
              </Switch.Control>
            </Switch>
          </SettingItem>

          <Separator />

          {/* 自动关闭 */}
          <SettingItem title={t("settings.review.autoClose.title")} description={t("settings.review.autoClose.desc")}>
            <Switch
              isSelected={autoClose}
              onChange={(val) => updateSetting("autoClose", String(val))}
            >
              <Switch.Control>
                <Switch.Thumb />
              </Switch.Control>
            </Switch>
          </SettingItem>
        </Card.Content>
      </Card>
    </div>
  );
}
