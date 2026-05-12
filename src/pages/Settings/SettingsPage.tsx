import { useEffect } from "react";
import { Tabs, Surface, Button } from "@heroui/react";
import { useSettingsStore } from "../../stores/settingsStore";
import { getSettings } from "../../lib/api";
import {
  Settings,
  Languages,
  BookOpen,
  Bell,
  Palette,
  Keyboard,
  Info,
  ArrowLeft,
} from "lucide-react";
import { GeneralPanel } from "./panels/GeneralPanel";
import { TranslationPanel } from "./panels/TranslationPanel";
import { ReviewPanel } from "./panels/ReviewPanel";
import { NotificationPanel } from "./panels/NotificationPanel";
import { AppearancePanel } from "./panels/AppearancePanel";
import { ShortcutsPanel } from "./panels/ShortcutsPanel";
import { AboutPanel } from "./panels/AboutPanel";
import { useNavigate } from "react-router-dom";

const tabs = [
  { id: "general", label: "通用", icon: Settings, panel: GeneralPanel },
  {
    id: "translation",
    label: "翻译",
    icon: Languages,
    panel: TranslationPanel,
  },
  { id: "learning", label: "学习", icon: BookOpen, panel: ReviewPanel },
  { id: "notification", label: "通知", icon: Bell, panel: NotificationPanel },
  { id: "appearance", label: "外观", icon: Palette, panel: AppearancePanel },
  { id: "shortcuts", label: "快捷键", icon: Keyboard, panel: ShortcutsPanel },
  { id: "about", label: "关于", icon: Info, panel: AboutPanel },
];

export function SettingsPage() {
  const navigate = useNavigate();
  const setSettings = useSettingsStore((state) => state.setSettings);

  useEffect(() => {
    getSettings().then((settings) => {
      setSettings(settings);
    });
  }, [setSettings]);

  return (
    <Surface className="flex h-screen animate-fade-in p-2 bg-background">
      <Tabs className="w-full" aria-label="设置导航">
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => navigate(-1)}>
            <ArrowLeft size={18} />
          </Button>
          <Tabs.ListContainer className="flex-1">
            <Tabs.List aria-label="设置分类">
              {tabs.map((tab) => (
                <Tabs.Tab key={tab.id} id={tab.id}>
                  <div className="flex items-center gap-3">
                    <tab.icon size={18} />
                    <span>{tab.label}</span>
                  </div>
                  <Tabs.Indicator />
                </Tabs.Tab>
              ))}
            </Tabs.List>
          </Tabs.ListContainer>
        </div>
        {tabs.map((tab) => (
          <Tabs.Panel className="p-1" key={tab.id} id={tab.id}>
            <tab.panel />
          </Tabs.Panel>
        ))}
      </Tabs>
    </Surface>
  );
}
