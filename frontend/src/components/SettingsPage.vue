<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import ServerList from "./ServerList.vue";
import type { ServerItem } from "./ServerItem.vue";

type SettingsTab = "general" | "server" | "kernel" | "advanced" | "about";
type Theme = "light" | "dark" | "system";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { t, locale } = useI18n();

const activeTab = ref<SettingsTab>("general");
const language = ref(locale.value);
const theme = ref<Theme>((localStorage.getItem("theme") as Theme) || "system");

const servers = ref<ServerItem[]>([
  {
    id: "1",
    title: "主服务器",
    enable: true,
    sort: 1,
    serverAddr: "192.168.1.100",
    serverPort: 7000,
    auth: {
      method: "token",
      token: "my-secret-token",
    },
  },
  {
    id: "2",
    title: "备用服务器",
    enable: false,
    sort: 2,
    serverAddr: "10.0.0.1",
    serverPort: 7000,
    auth: null,
  },
]);

const languages = [
  { value: "zh-CN", label: "简体中文" },
  { value: "zh-TW", label: "繁體中文" },
  { value: "en", label: "English" },
  { value: "ja", label: "日本語" },
];

const themes = [
  { value: "light" as Theme, labelKey: "settings.theme.light", icon: "sun" },
  { value: "dark" as Theme, labelKey: "settings.theme.dark", icon: "moon" },
  { value: "system" as Theme, labelKey: "settings.theme.system", icon: "monitor" },
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

function handleUpdateServers(newItems: ServerItem[]) {
  servers.value = newItems;
}

function handleSaveServer(id: string, data: ServerItem) {
  const index = servers.value.findIndex((s) => s.id === id);
  if (index !== -1) {
    servers.value[index] = data;
  }
}

function handleDeleteServer(id: string) {
  servers.value = servers.value.filter((s) => s.id !== id);
}

function handleAddServer() {
  const newServer: ServerItem = {
    id: String(Date.now()),
    title: "新服务器",
    enable: false,
    sort: servers.value.length + 1,
    serverAddr: "127.0.0.1",
    serverPort: 7000,
    auth: null,
  };
  servers.value.push(newServer);
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
    <header class="flex items-center gap-3 px-6 py-4 border-b border-border">
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

    <div class="flex gap-1 p-1 mx-6 mt-4 border border-border rounded-lg bg-muted/30">
      <button
        class="flex-1 px-3 py-1.5 rounded-md text-sm font-medium transition-colors"
        :class="activeTab === 'general' ? 'bg-primary text-primary-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'"
        @click="activeTab = 'general'"
      >
        {{ t('settings.tabs.general') }}
      </button>
      <button
        class="flex-1 px-3 py-1.5 rounded-md text-sm font-medium transition-colors"
        :class="activeTab === 'server' ? 'bg-primary text-primary-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'"
        @click="activeTab = 'server'"
      >
        {{ t('settings.tabs.server') }}
      </button>
      <button
        class="flex-1 px-3 py-1.5 rounded-md text-sm font-medium transition-colors"
        :class="activeTab === 'kernel' ? 'bg-primary text-primary-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'"
        @click="activeTab = 'kernel'"
      >
        {{ t('settings.tabs.kernel') }}
      </button>
      <button
        class="flex-1 px-3 py-1.5 rounded-md text-sm font-medium transition-colors"
        :class="activeTab === 'advanced' ? 'bg-primary text-primary-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'"
        @click="activeTab = 'advanced'"
      >
        {{ t('settings.tabs.advanced') }}
      </button>
      <button
        class="flex-1 px-3 py-1.5 rounded-md text-sm font-medium transition-colors"
        :class="activeTab === 'about' ? 'bg-primary text-primary-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'"
        @click="activeTab = 'about'"
      >
        {{ t('settings.tabs.about') }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-6 py-6">
      <div v-if="activeTab === 'general'" class="space-y-6">
        <section class="space-y-3">
          <header class="space-y-1">
            <h3 class="text-sm font-medium">{{ t('settings.language.title') }}</h3>
            <p class="text-xs text-muted-foreground">{{ t('settings.language.description') }}</p>
          </header>
          <div class="inline-flex gap-1 rounded-md border border-border bg-background p-1">
            <button
              v-for="lang in languages"
              :key="lang.value"
              class="inline-flex items-center justify-center min-w-[96px] px-3 py-1.5 rounded-md text-sm font-medium transition-colors"
              :class="language === lang.value ? 'bg-primary text-primary-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-muted'"
              @click="language = lang.value"
            >
              {{ lang.label }}
            </button>
          </div>
        </section>

        <section class="space-y-3">
          <header class="space-y-1">
            <h3 class="text-sm font-medium">{{ t('settings.theme.title') }}</h3>
            <p class="text-xs text-muted-foreground">{{ t('settings.theme.description') }}</p>
          </header>
          <div class="inline-flex gap-1 rounded-md border border-border bg-background p-1">
            <button
              v-for="th in themes"
              :key="th.value"
              class="inline-flex items-center justify-center gap-1.5 min-w-[96px] px-3 py-1.5 rounded-md text-sm font-medium transition-colors"
              :class="theme === th.value ? 'bg-primary text-primary-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-muted'"
              @click="theme = th.value"
            >
              <svg v-if="th.icon === 'sun'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="4" />
                <path d="M12 2v2" />
                <path d="M12 20v2" />
                <path d="m4.93 4.93 1.41 1.41" />
                <path d="m17.66 17.66 1.41 1.41" />
                <path d="M2 12h2" />
                <path d="M20 12h2" />
                <path d="m6.34 17.66-1.41 1.41" />
                <path d="m19.07 4.93-1.41 1.41" />
              </svg>
              <svg v-else-if="th.icon === 'moon'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
              </svg>
              <svg v-else-if="th.icon === 'monitor'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect width="20" height="14" x="2" y="3" rx="2" />
                <line x1="8" x2="16" y1="21" y2="21" />
                <line x1="12" x2="12" y1="17" y2="21" />
              </svg>
              {{ t(th.labelKey) }}
            </button>
          </div>
        </section>
      </div>

      <div v-else-if="activeTab === 'server'" class="space-y-6">
        <ServerList
          :items="servers"
          @update:items="handleUpdateServers"
          @save="handleSaveServer"
          @delete="handleDeleteServer"
          @add="handleAddServer"
        />
      </div>

      <div v-else-if="activeTab === 'kernel'" class="space-y-6">
        <section class="space-y-3">
          <header class="space-y-1">
            <h3 class="text-sm font-medium">{{ t('settings.tabs.kernel') }}</h3>
            <p class="text-xs text-muted-foreground">Kernel settings coming soon.</p>
          </header>
        </section>
      </div>

      <div v-else-if="activeTab === 'advanced'" class="space-y-6">
        <section class="space-y-3">
          <header class="space-y-1">
            <h3 class="text-sm font-medium">{{ t('settings.tabs.advanced') }}</h3>
            <p class="text-xs text-muted-foreground">Advanced settings coming soon.</p>
          </header>
        </section>
      </div>

      <div v-else-if="activeTab === 'about'" class="space-y-6">
        <section class="space-y-3">
          <header class="space-y-1">
            <h3 class="text-sm font-medium">{{ t('settings.tabs.about') }}</h3>
            <p class="text-xs text-muted-foreground">frpc tray v0.1.0</p>
          </header>
        </section>
      </div>
    </div>
  </div>
</template>
