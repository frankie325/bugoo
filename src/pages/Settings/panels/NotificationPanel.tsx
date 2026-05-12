import { useSettingsStore } from "../../../stores/settingsStore";
import { Card, Separator, CheckboxGroup, Checkbox, Input } from "@heroui/react";
import { SettingItem } from "../components/SettingItem";

export function NotificationPanel() {
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
        <Card.Title>通知设置</Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 提醒时间 */}
        <SettingItem title="提醒时间" description="设置每日提醒时间段">
          <div className="flex items-center gap-2">
            <Input
              type="time"
              value={reminderStartTime}
              onChange={(e) =>
                updateSetting("reminderStartTime", e.target.value)
              }
              className="w-36"
            />
            <span className="text-default-500">—</span>
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
        <SettingItem title="通知类型" description="选择需要接收的通知类型">
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
              每日复习提醒
            </Checkbox>
            <Checkbox value="forgetting">
              <Checkbox.Control>
                <Checkbox.Indicator />
              </Checkbox.Control>
              即将遗忘提醒
            </Checkbox>
            <Checkbox value="streak">
              <Checkbox.Control>
                <Checkbox.Indicator />
              </Checkbox.Control>
              连续学习提醒
            </Checkbox>
            <Checkbox value="achievement">
              <Checkbox.Control>
                <Checkbox.Indicator />
              </Checkbox.Control>
              成就通知
            </Checkbox>
          </CheckboxGroup>
        </SettingItem>
      </Card.Content>
    </Card>
  );
}
