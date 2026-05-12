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

const languageOptions = [
  { label: "简体中文", value: "zh-CN" },
  { label: "繁體中文", value: "zh-TW" },
  { label: "English", value: "en" },
  { label: "日本語", value: "ja" },
];

const themeColors = [
  { label: "绿色", color: "#10b981" },
  { label: "蓝色", color: "#3b82f6" },
  { label: "紫色", color: "#8b5cf6" },
];

export function GeneralPanel() {
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);

  const language = settings.language || "zh-CN";
  const startup = settings.startup === "true";
  const closeBehavior = settings.closeBehavior || "minimize";
  const autoUpdate = settings.autoUpdate !== "false";
  const theme = settings.theme || "#10b981";

  return (
    <Card>
      <Card.Header>
        <Card.Title>
          <Text type="h3">通用设置</Text>
        </Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 主题色 */}
        <SettingItem title="主题颜色" description="选择应用主色调">
          <ColorSwatchPicker
            value={theme}
            onChange={(color) =>
              updateSetting("theme", color as unknown as string)
            }
          >
            {themeColors.map((c) => (
              <ColorSwatchPicker.Item key={c.label} color={c.color}>
                <ColorSwatchPicker.Swatch color={c.color} />
              </ColorSwatchPicker.Item>
            ))}
          </ColorSwatchPicker>
        </SettingItem>

        <Separator />

        {/* 启动时打开 */}
        <SettingItem title="启动时打开" description="应用启动时自动打开主窗口">
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
        <SettingItem title="自动更新" description="检查并自动安装更新">
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
        <SettingItem title="关闭行为" description="关闭主窗口时的操作">
          <RadioGroup
            orientation="horizontal"
            value={closeBehavior}
            onChange={(val) => updateSetting("closeBehavior", val)}
          >
            <Radio value="minimize">
              <Radio.Control>
                <Radio.Indicator />
              </Radio.Control>
              最小化到后台
            </Radio>
            <Radio value="quit">
              <Radio.Control>
                <Radio.Indicator />
              </Radio.Control>
              退出应用
            </Radio>
          </RadioGroup>
        </SettingItem>

        <Separator />

        {/* 界面语言 */}
        <SettingItem title="界面语言" description="选择应用界面语言">
          <Select
            className="w-36"
            value={language}
            onChange={(value) =>
              value && updateSetting("language", String(value))
            }
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
