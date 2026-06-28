import { ref, watch } from "vue";
import { getConfig, saveConfig as saveConfigToBackend } from "./ipc";

type Theme = "light" | "dark" | "system";

export interface AppConfig {
  language: string;
  theme: Theme;
  autostart: boolean;
  silentLaunch: boolean;
  autoRun: boolean;
  useGithubProxy: boolean;
}

const defaultConfig: AppConfig = {
  language: "zh-CN",
  theme: "system",
  autostart: false,
  silentLaunch: false,
  autoRun: false,
  useGithubProxy: false,
};

function toBackendConfig(config: AppConfig) {
  return {
    language: config.language,
    theme: config.theme,
    autostart: config.autostart,
    silent_launch: config.silentLaunch,
    auto_run: config.autoRun,
    use_github_proxy: config.useGithubProxy,
  };
}

function fromBackendConfig(data: any): AppConfig {
  return {
    language: data.language || defaultConfig.language,
    theme: data.theme || defaultConfig.theme,
    autostart: data.autostart ?? defaultConfig.autostart,
    silentLaunch: data.silent_launch ?? defaultConfig.silentLaunch,
    autoRun: data.auto_run ?? defaultConfig.autoRun,
    useGithubProxy: data.use_github_proxy ?? defaultConfig.useGithubProxy,
  };
}

export const appConfig = ref<AppConfig>({ ...defaultConfig });
export const configLoaded = ref(false);

export async function loadConfig() {
  try {
    const backendConfig = await getConfig();
    appConfig.value = fromBackendConfig(backendConfig);
  } catch (e) {
    console.error("Failed to load config from backend:", e);
    appConfig.value = { ...defaultConfig };
  } finally {
    configLoaded.value = true;
  }
}

export async function updateConfig(patch: Partial<AppConfig>) {
  Object.assign(appConfig.value, patch);
  try {
    await saveConfigToBackend(toBackendConfig(appConfig.value));
  } catch (e) {
    console.error("Failed to save config to backend:", e);
  }
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
