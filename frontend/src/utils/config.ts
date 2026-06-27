import { ref, watch } from "vue";

type Theme = "light" | "dark" | "system";

interface AppConfig {
  language: string;
  theme: Theme;
  autostart: boolean;
  silentLaunch: boolean;
  autoRun: boolean;
}

const STORAGE_KEY = "app_config";

const defaultConfig: AppConfig = {
  language: "zh-CN",
  theme: "system",
  autostart: false,
  silentLaunch: false,
  autoRun: false,
};

function loadConfig(): AppConfig {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      return { ...defaultConfig, ...JSON.parse(raw) };
    }
  } catch {}
  return { ...defaultConfig };
}

function saveConfig(config: AppConfig) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
}

export const appConfig = ref<AppConfig>(loadConfig());

export function updateConfig(patch: Partial<AppConfig>) {
  Object.assign(appConfig.value, patch);
  saveConfig(appConfig.value);
}

export function initConfigWatchers() {
  watch(
    () => appConfig.value.theme,
    (newTheme) => {
      const root = document.documentElement;
      if (newTheme === "system") {
        const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
        root.classList.toggle("dark", prefersDark);
        root.classList.toggle("light", !prefersDark);
      } else if (newTheme === "dark") {
        root.classList.add("dark");
        root.classList.remove("light");
      } else {
        root.classList.add("light");
        root.classList.remove("dark");
      }
    },
    { immediate: true }
  );
}
