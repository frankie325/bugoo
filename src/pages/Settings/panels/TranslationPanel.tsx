import { useSettingsStore } from "../../../stores/settingsStore";
import { Card, Select, ListBox, Input, Separator, Label } from "@heroui/react";
import { SettingItem } from "../components/SettingItem";

const engineOptions = [
  { label: "DeepL", value: "deepl" },
  { label: "Google", value: "google" },
  { label: "OpenAI", value: "openai" },
  { label: "自定义", value: "custom" },
];

export function TranslationPanel() {
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);

  const translationEngine = settings.translationEngine || "deepl";
  const apiEndpoint = settings.apiEndpoint || "";
  const apiKey = settings.apiKey || "";

  return (
    <Card>
      <Card.Header>
        <Card.Title>翻译设置</Card.Title>
      </Card.Header>
      <Card.Content>
        {/* 翻译引擎 */}
        <SettingItem title="翻译引擎" description="选择默认翻译服务">
          <Select
            className="w-40"
            value={translationEngine}
            onChange={(value) =>
              value && updateSetting("translationEngine", String(value))
            }
          >
            <Label>翻译引擎</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {engineOptions.map((opt) => (
                  <ListBox.Item
                    key={opt.value}
                    id={opt.value}
                    textValue={opt.label}
                  >
                    {opt.label}
                    <ListBox.ItemIndicator />
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>
        </SettingItem>

        <Separator />

        {/* API地址 */}
        <SettingItem title="API地址" description="翻译服务的接口地址">
          <Input
            value={apiEndpoint}
            onChange={(e) => updateSetting("apiEndpoint", e.target.value)}
            placeholder="https://api.example.com"
            className="w-64"
          />
        </SettingItem>

        <Separator />

        {/* API秘钥 */}
        <SettingItem title="API秘钥" description="翻译服务的认证密钥">
          <Input
            value={apiKey}
            onChange={(e) => updateSetting("apiKey", e.target.value)}
            placeholder="sk-xxx"
            type="password"
            className="w-64"
          />
        </SettingItem>
      </Card.Content>
    </Card>
  );
}
