<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";

type SettingsTab = "general" | "advanced" | "about";
type Theme = "light" | "dark" | "system";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { t, locale } = useI18n();

const activeTab = ref<SettingsTab>("general");
const language = ref(locale.value);
const theme = ref<Theme>((localStorage.getItem("theme") as Theme) || "system");

const languages = [
  { value: "zh-CN", label: "简体中文" },
  { value: "zh-TW", label: "繁體中文" },
  { value: "en", label: "English" },
  { value: "ja", label: "日本語" },
];

const themes = [
  { value: "light" as Theme, labelKey: "settings.theme.light" },
  { value: "dark" as Theme, labelKey: "settings.theme.dark" },
  { value: "system" as Theme, labelKey: "settings.theme.system" },
];

function applyTheme(newTheme: Theme) {
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
}

watch(theme, (newTheme) => {
  applyTheme(newTheme);
  localStorage.setItem("theme", newTheme);
});

watch(language, (newLang) => {
  locale.value = newLang;
  localStorage.setItem("language", newLang);
});
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="flex items-center gap-3 px-4 py-3 border-b border-border">
      <button
        class="inline-flex items-center justify-center h-8 w-8 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        @click="emit('close')"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="m12 19-7-7 7-7" />
          <path d="M19 12H5" />
        </svg>
      </button>
      <h1 class="text-lg font-semibold">{{ t('settings.title') }}</h1>
    </header>

    <div class="flex gap-1 p-2 border-b border-border">
      <button
        class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
        :class="activeTab === 'general' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted hover:text-foreground'"
        @click="activeTab = 'general'"
      >
        {{ t('settings.tabs.general') }}
      </button>
      <button
        class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
        :class="activeTab === 'advanced' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted hover:text-foreground'"
        @click="activeTab = 'advanced'"
      >
        {{ t('settings.tabs.advanced') }}
      </button>
      <button
        class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
        :class="activeTab === 'about' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted hover:text-foreground'"
        @click="activeTab = 'about'"
      >
        {{ t('settings.tabs.about') }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-4">
      <div v-if="activeTab === 'general'" class="space-y-6">
        <div>
          <h3 class="text-sm font-medium mb-2">{{ t('settings.language.title') }}</h3>
          <p class="text-xs text-muted-foreground mb-3">{{ t('settings.language.description') }}</p>
          <div class="flex gap-2">
            <button
              v-for="lang in languages"
              :key="lang.value"
              class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
              :class="language === lang.value ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground hover:bg-muted/80'"
              @click="language = lang.value"
            >
              {{ lang.label }}
            </button>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium mb-2">{{ t('settings.theme.title') }}</h3>
          <p class="text-xs text-muted-foreground mb-3">{{ t('settings.theme.description') }}</p>
          <div class="flex gap-2">
            <button
              v-for="th in themes"
              :key="th.value"
              class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
              :class="theme === th.value ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground hover:bg-muted/80'"
              @click="theme = th.value"
            >
              {{ t(th.labelKey) }}
            </button>
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'advanced'" class="space-y-6">
        <div>
          <h3 class="text-sm font-medium mb-2">{{ t('settings.tabs.advanced') }}</h3>
          <p class="text-xs text-muted-foreground">Advanced settings coming soon.</p>
        </div>
      </div>

      <div v-else-if="activeTab === 'about'" class="space-y-6">
        <div>
          <h3 class="text-sm font-medium mb-2">{{ t('settings.tabs.about') }}</h3>
          <p class="text-xs text-muted-foreground">frpc tray v0.1.0</p>
        </div>
      </div>
    </div>
  </div>
</template>
