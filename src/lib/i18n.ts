import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import zhCN from "../locales/zh-CN/common.json";
import zhTW from "../locales/zh-TW/common.json";
import en from "../locales/en/common.json";
import ja from "../locales/ja/common.json";
import ko from "../locales/ko/common.json";
import es from "../locales/es/common.json";
import fr from "../locales/fr/common.json";
import de from "../locales/de/common.json";
import pt from "../locales/pt/common.json";
import ru from "../locales/ru/common.json";
import ar from "../locales/ar/common.json";
import hi from "../locales/hi/common.json";
import th from "../locales/th/common.json";
import vi from "../locales/vi/common.json";
import id from "../locales/id/common.json";

const resources = {
  "zh-CN": { common: zhCN },
  "zh-TW": { common: zhTW },
  en: { common: en },
  ja: { common: ja },
  ko: { common: ko },
  es: { common: es },
  fr: { common: fr },
  de: { common: de },
  pt: { common: pt },
  ru: { common: ru },
  ar: { common: ar },
  hi: { common: hi },
  th: { common: th },
  vi: { common: vi },
  id: { common: id },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: "en",
    defaultNS: "common",
    ns: ["common"],
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ["querystring", "localStorage", "navigator"],
      lookupQuerystring: "lang",
      lookupLocalStorage: "bugoo-language",
    },
  });

export default i18n;