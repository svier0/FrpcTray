import { createApp } from "vue";
import App from "./App.vue";
import i18n from "./i18n";
import "./style.css";
import { initConfigWatchers } from "./utils/config";

function initTheme() {
  let theme: "light" | "dark" | "system" = "system";
  try {
    const raw = localStorage.getItem("app_config");
    if (raw) {
      theme = JSON.parse(raw).theme || "system";
    }
  } catch {
    theme = (localStorage.getItem("theme") as "light" | "dark" | "system") || "system";
  }
  
  const root = document.documentElement;
  
  if (theme === "system") {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.classList.toggle("dark", prefersDark);
    root.classList.toggle("light", !prefersDark);
  } else if (theme === "dark") {
    root.classList.add("dark");
    root.classList.remove("light");
  } else {
    root.classList.add("light");
    root.classList.remove("dark");
  }
}

initTheme();
initConfigWatchers();

createApp(App).use(i18n).mount("#app");
