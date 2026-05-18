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
import { useTranslation } from "react-i18next";

const tabConfig = [
  { id: "general", icon: Settings, panel: GeneralPanel },
  { id: "translation", icon: Languages, panel: TranslationPanel },
  { id: "learning", icon: BookOpen, panel: ReviewPanel },
  { id: "notification", icon: Bell, panel: NotificationPanel },
  { id: "appearance", icon: Palette, panel: AppearancePanel },
  { id: "shortcuts", icon: Keyboard, panel: ShortcutsPanel },
  { id: "about", icon: Info, panel: AboutPanel },
];

export function SettingsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const setSettings = useSettingsStore((state) => state.setSettings);

  useEffect(() => {
    getSettings().then((settings) => {
      setSettings(settings);
    });
  }, [setSettings]);

  return (
    <Surface className="flex h-screen animate-fade-in p-2 bg-background">
      <Tabs className="w-full" aria-label={t("settings.ariaLabel")}>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => navigate(-1)}>
            <ArrowLeft size={18} />
          </Button>
          <Tabs.ListContainer className="flex-1">
            <Tabs.List aria-label={t("settings.listAriaLabel")}>
              {tabConfig.map((tab) => (
                <Tabs.Tab key={tab.id} id={tab.id}>
                  <div className="flex items-center gap-3">
                    <tab.icon size={18} />
                    <span>{t(`settings.tabs.${tab.id}`)}</span>
                  </div>
                  <Tabs.Indicator />
                </Tabs.Tab>
              ))}
            </Tabs.List>
          </Tabs.ListContainer>
        </div>
        {tabConfig.map((tab) => (
          <Tabs.Panel className="p-1" key={tab.id} id={tab.id}>
            <tab.panel />
          </Tabs.Panel>
        ))}
      </Tabs>
    </Surface>
  );
}
