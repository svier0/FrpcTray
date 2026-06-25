import { createApp } from "vue";
import App from "./App.vue";
import i18n from "./i18n";
import "./style.css";

function initTheme() {
  const theme = (localStorage.getItem("theme") as "light" | "dark" | "system") || "system";
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

createApp(App).use(i18n).mount("#app");
