import { useEffect } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { HomePage } from "./pages/Home/HomePage";
import { SettingsPage } from "./pages/Settings/SettingsPage";
import { seedSettings, getSettings } from "./lib/api";
import { useSettingsStore } from "./stores/settingsStore";

function App() {
  const setSettings = useSettingsStore((state) => state.setSettings);

  useEffect(() => {
    seedSettings().then(() => {
      getSettings().then((settings) => {
        setSettings(settings);
      });
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
