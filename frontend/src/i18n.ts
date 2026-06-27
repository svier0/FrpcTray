import { createI18n } from "vue-i18n";
import zhCN from "./locales/zh-CN.json";
import zhTW from "./locales/zh-TW.json";
import en from "./locales/en.json";
import ja from "./locales/ja.json";

function getLanguage(): string {
  try {
    const raw = localStorage.getItem("app_config");
    if (raw) {
      return JSON.parse(raw).language || "zh-CN";
    }
  } catch {}
  return localStorage.getItem("language") || "zh-CN";
}

const i18n = createI18n({
  legacy: false,
  locale: getLanguage(),
  fallbackLocale: "zh-CN",
  messages: {
    "zh-CN": zhCN,
    "zh-TW": zhTW,
    en: en,
    ja: ja,
  },
});

export default i18n;
