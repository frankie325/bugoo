import { Button } from "@heroui/react";
import { useTranslation } from "react-i18next";

export function AboutPanel() {
  const { t } = useTranslation();
  const version = "1.0.0";

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium mb-4">{t("settings.about.title")}</h2>
        <div className="space-y-6">
          <div className="flex flex-col items-center text-center py-6">
            <div className="w-16 h-16 bg-primary rounded-2xl flex items-center justify-center mb-4">
              <span className="text-2xl text-white">B</span>
            </div>
            <h3 className="text-xl font-semibold">{t("app.name")}</h3>
            <p className="text-sm text-default-500 mt-1">{t("settings.about.version", { version })}</p>
            <p className="text-xs text-default-400 mt-2">
              {t("settings.about.tagline")}
            </p>
          </div>

          <div className="space-y-3">
            <Button
              variant="outline"
              className="w-full justify-start"
              onPress={() => {}}
            >
              {t("settings.about.checkUpdate")}
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onPress={() => {}}
            >
              {t("settings.about.feedback")}
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onPress={() => {}}
            >
              {t("settings.about.helpDoc")}
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onPress={() => {}}
            >
              {t("settings.about.license")}
            </Button>
          </div>

          <div className="pt-4 border-t border-default-200">
            <p className="text-xs text-default-400 text-center">
              {t("settings.about.footer")}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
