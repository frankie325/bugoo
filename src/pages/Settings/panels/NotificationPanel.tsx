import { useSettingsStore } from "../../../stores/settingsStore";
import { Card, Separator, CheckboxGroup, Checkbox, Input } from "@heroui/react";
import { SettingItem } from "../components/SettingItem";
import { useTranslation } from "react-i18next";

export function NotificationPanel() {
  const { t } = useTranslation();
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);

  const reminderStartTime = settings.reminderStartTime || "09:00";
  const reminderEndTime = settings.reminderEndTime || "21:00";
  const notifyDailyReview = settings.notifyDailyReview !== "false";
  const notifyForgetting = settings.notifyForgetting !== "false";
  const notifyStreak = settings.notifyStreak !== "false";
  const notifyAchievement = settings.notifyAchievement !== "false";

  const notificationTypes = [
    notifyDailyReview && "daily",
    notifyForgetting && "forgetting",
    notifyStreak && "streak",
    notifyAchievement && "achievement",
  ].filter(Boolean) as string[];

  return (
    <Card>
      <Card.Header>
        <Card.Title>{t("settings.notification.title")}</Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 提醒时间 */}
        <SettingItem title={t("settings.notification.reminderTime.title")} description={t("settings.notification.reminderTime.desc")}>
          <div className="flex items-center gap-2">
            <Input
              type="time"
              value={reminderStartTime}
              onChange={(e) =>
                updateSetting("reminderStartTime", e.target.value)
              }
              className="w-36"
            />
            <span className="text-default-500">{t("settings.notification.timeDash")}</span>
            <Input
              type="time"
              value={reminderEndTime}
              onChange={(e) => updateSetting("reminderEndTime", e.target.value)}
              className="w-36"
            />
          </div>
        </SettingItem>

        <Separator />

        {/* 通知类型 */}
        <SettingItem title={t("settings.notification.types.title")} description={t("settings.notification.types.desc")}>
          <CheckboxGroup
            className="flex flex-row gap-4"
            value={notificationTypes}
            onChange={(vals) => {
              updateSetting(
                "notifyDailyReview",
                String(vals.includes("daily")),
              );
              updateSetting(
                "notifyForgetting",
                String(vals.includes("forgetting")),
              );
              updateSetting("notifyStreak", String(vals.includes("streak")));
              updateSetting(
                "notifyAchievement",
                String(vals.includes("achievement")),
              );
            }}
          >
            <Checkbox value="daily">
              <Checkbox.Control>
                <Checkbox.Indicator />
              </Checkbox.Control>
              {t("settings.notification.dailyReview")}
            </Checkbox>
            <Checkbox value="forgetting">
              <Checkbox.Control>
                <Checkbox.Indicator />
              </Checkbox.Control>
              {t("settings.notification.forgetting")}
            </Checkbox>
            <Checkbox value="streak">
              <Checkbox.Control>
                <Checkbox.Indicator />
              </Checkbox.Control>
              {t("settings.notification.streak")}
            </Checkbox>
            <Checkbox value="achievement">
              <Checkbox.Control>
                <Checkbox.Indicator />
              </Checkbox.Control>
              {t("settings.notification.achievement")}
            </Checkbox>
          </CheckboxGroup>
        </SettingItem>
      </Card.Content>
    </Card>
  );
}
