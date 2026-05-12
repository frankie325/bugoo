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

const themeColors = [
  { label: "绿色", color: "#10b981" },
  { label: "蓝色", color: "#3b82f6" },
  { label: "紫色", color: "#8b5cf6" },
];

const cardStyleOptions = [
  { label: "简洁", value: "simple" },
  { label: "丰富", value: "rich" },
];

const fontSizeOptions = [
  { label: "小", value: "small" },
  { label: "中", value: "medium" },
  { label: "大", value: "large" },
];

export function AppearancePanel() {
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
        <Card.Title>外观与个性化</Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 主题 */}
        <SettingItem title="主题" description="选择应用配色方案">
          <Tabs
            selectedKey={theme}
            onSelectionChange={(key) => updateSetting("theme", key as string)}
          >
            <Tabs.ListContainer>
              <Tabs.List aria-label="主题">
                <Tabs.Tab id="light">
                  浅色
                  <Tabs.Indicator />
                </Tabs.Tab>
                <Tabs.Tab id="dark">
                  深色
                  <Tabs.Indicator />
                </Tabs.Tab>
                <Tabs.Tab id="system">
                  跟随系统
                  <Tabs.Indicator />
                </Tabs.Tab>
              </Tabs.List>
            </Tabs.ListContainer>
          </Tabs>
        </SettingItem>

        <Separator />

        {/* 主题颜色 */}
        <SettingItem title="主题颜色" description="选择应用主色调">
          <ColorSwatchPicker
            value={colorValue}
            onChange={(color) =>
              updateSetting("themeColor", color.toString("hex"))
            }
          >
            {themeColors.map((c) => (
              <ColorSwatchPicker.Item key={c.label} color={c.color}>
                <ColorSwatchPicker.Swatch color={c.color} />
                <ColorSwatchPicker.Indicator />
              </ColorSwatchPicker.Item>
            ))}
          </ColorSwatchPicker>
        </SettingItem>

        <Separator />

        {/* 卡片风格 */}
        <SettingItem title="卡片风格" description="单词卡片的显示风格">
          <RadioGroup
            className="flex flex-row gap-4"
            value={cardStyle}
            onChange={(val) => updateSetting("cardStyle", val)}
          >
            {cardStyleOptions.map((opt) => (
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

        {/* 字体大小 */}
        <SettingItem title="字体大小" description="选择应用字体大小">
          <Tabs
            selectedKey={fontSize}
            onSelectionChange={(key) =>
              updateSetting("fontSize", key as string)
            }
          >
            <Tabs.ListContainer>
              <Tabs.List aria-label="字体大小">
                {fontSizeOptions.map((opt) => (
                  <Tabs.Tab key={opt.value} id={opt.value}>
                    {opt.label}
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
