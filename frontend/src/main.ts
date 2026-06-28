import { createApp } from "vue";
import App from "./App.vue";
import i18n from "./i18n";
import "./style.css";
import { loadConfig, initConfigWatchers } from "./utils/config";
import { invoke } from "@tauri-apps/api/core";

(window as any).cmd = (name: string, args?: Record<string, unknown>) => invoke(name, args);

async function init() {
  await loadConfig();
  initConfigWatchers();
  createApp(App).use(i18n).mount("#app");
}

init();
