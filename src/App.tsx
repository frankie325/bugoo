import { useEffect } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { HomePage } from "./pages/Home/HomePage";
import { SettingsPage } from "./pages/Settings/SettingsPage";
import { seedSettings, getSettings } from "./lib/api";
import { useSettingsStore } from "./stores/settingsStore";
import "./lib/i18n"; // i18next 初始化
import i18n from "i18next";

function App() {
  const setSettings = useSettingsStore((state) => state.setSettings);

  useEffect(() => {
    seedSettings().then(async () => {
      const settings = await getSettings();
      setSettings(settings);
      const lang = settings.language || "en";
      await i18n.changeLanguage(lang);
    });
  }, [setSettings]);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
