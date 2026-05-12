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

const dailyLimitOptions = [
  { label: "10", value: "10" },
  { label: "20", value: "20" },
  { label: "50", value: "50" },
  { label: "自定义", value: "custom" },
];

const reviewPaceOptions = [
  { label: "轻松", value: "relaxed" },
  { label: "标准", value: "normal" },
  { label: "强化", value: "intense" },
];

const hintStrategyOptions = [
  { label: "渐进提示", value: "progressive" },
  { label: "直接显示答案", value: "immediate" },
];

export function ReviewPanel() {
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
          <Card.Title>复习设置</Card.Title>
        </Card.Header>
        <Card.Content>
          {/* 每日复习上限 */}
          <SettingItem title="每日复习上限" description="每天最多复习的单词数">
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
          <SettingItem title="复习节奏" description="根据掌握程度调整复习频率">
            <RadioGroup
              orientation="horizontal"
              value={reviewPace}
              onChange={(val) => updateSetting("reviewPace", val)}
            >
              {reviewPaceOptions.map((opt) => (
                <Radio key={opt.value} value={opt.value}>
                  <Radio.Control>
                    <Radio.Indicator />
                  </Radio.Control>
                  {opt.label}
                </Radio>
              ))}
            </RadioGroup>
          </SettingItem>

          <Separator />

          {/* 提示策略 */}
          <SettingItem title="提示策略" description="记忆时的提示方式">
            <RadioGroup
              orientation="horizontal"
              value={hintStrategy}
              onChange={(val) => updateSetting("hintStrategy", val)}
            >
              {hintStrategyOptions.map((opt) => (
                <Radio key={opt.value} value={opt.value}>
                  <Radio.Control>
                    <Radio.Indicator />
                  </Radio.Control>
                  {opt.label}
                </Radio>
              ))}
            </RadioGroup>
          </SettingItem>
        </Card.Content>
      </Card>

      {/* 划词设置 */}
      <Card>
        <Card.Header>
          <Card.Title>划词设置</Card.Title>
        </Card.Header>
        <Card.Content>
          {/* 是否启用划词翻译 */}
          <SettingItem
            title="启用划词翻译"
            description="选中文字时自动弹出翻译"
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
          <SettingItem title="自动发音" description="显示翻译后自动朗读单词">
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
          <SettingItem title="自动关闭" description="失去焦点时自动关闭弹窗">
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
