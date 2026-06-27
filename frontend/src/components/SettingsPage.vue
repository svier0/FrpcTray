<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import ServerList from "./ServerList.vue";
import type { ServerItem } from "./ServerItem.vue";
import ConfirmDialog from "./ConfirmDialog.vue";
import { listServers, createServer, updateServer, deleteServer, reorderServers, getFrpcVersion, upgradeFrpc } from "../utils/ipc";
import type { FrpcVersionInfo } from "../utils/ipc";

type SettingsTab = "general" | "server" | "kernel" | "advanced" | "about";
type Theme = "light" | "dark" | "system";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { t, locale } = useI18n();

const activeTab = ref<SettingsTab>("general");
const language = ref(locale.value);
const theme = ref<Theme>((localStorage.getItem("theme") as Theme) || "system");

const servers = ref<ServerItem[]>([]);

const showDeleteDialog = ref(false);
const deleteTargetId = ref<string | null>(null);
const isLoading = ref(false);

const versionInfo = ref<FrpcVersionInfo | null>(null);
const isUpgrading = ref(false);
const upgradeProgress = ref("");

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

async function loadServers() {
  try {
    isLoading.value = true;
    servers.value = await listServers();
  } catch (e) {
    console.error("Failed to load servers:", e);
  } finally {
    isLoading.value = false;
  }
}

async function handleUpdateServers(newItems: ServerItem[]) {
  servers.value = newItems;
  // 调用后端接口保存排序
  try {
    await reorderServers(newItems.map((s) => s.id));
  } catch (e) {
    console.error("Failed to reorder servers:", e);
  }
}

async function handleSaveServer(id: string, data: ServerItem) {
  try {
    await updateServer(data);
    const index = servers.value.findIndex((s) => s.id === id);
    if (index !== -1) {
      servers.value[index] = data;
    }
  } catch (e) {
    console.error("Failed to save server:", e);
  }
}

function handleDeleteServer(id: string) {
  deleteTargetId.value = id;
  showDeleteDialog.value = true;
}

async function confirmDelete() {
  if (!deleteTargetId.value) return;
  try {
    await deleteServer(deleteTargetId.value);
    servers.value = servers.value.filter((s) => s.id !== deleteTargetId.value);
  } catch (e) {
    console.error("Failed to delete server:", e);
  } finally {
    showDeleteDialog.value = false;
    deleteTargetId.value = null;
  }
}

function cancelDelete() {
  showDeleteDialog.value = false;
  deleteTargetId.value = null;
}

async function handleAddServer() {
  const usedIds = servers.value.map((s) => s.id);
  const allLetters = "abcdefghijklmnopqrstuvwxyz".split("");
  const nextId = allLetters.find((letter) => !usedIds.includes(letter));

  if (!nextId) {
    console.error("Maximum 26 servers reached");
    return;
  }

  const newServerData = {
    title: "新服务器",
    enable: false,
    sort: servers.value.length + 1,
    serverAddr: "127.0.0.1",
    serverPort: 7000,
    auth: {
      method: "token",
    },
  };
  try {
    const id = await createServer(newServerData);
    const newServer: ServerItem = {
      id,
      ...newServerData,
    };
    servers.value.push(newServer);
  } catch (e) {
    console.error("Failed to create server:", e);
  }
}

onMounted(() => {
  loadServers();
  loadVersionInfo();
});

async function loadVersionInfo() {
  try {
    versionInfo.value = await getFrpcVersion();
  } catch (e) {
    console.error("Failed to load version info:", e);
  }
}

async function handleUpgrade() {
  if (!versionInfo.value || isUpgrading.value) return;
  const isInstall = versionInfo.value.current_version === '0';
  try {
    isUpgrading.value = true;
    upgradeProgress.value = isInstall ? t('settings.kernel.installing') : t('settings.kernel.upgrading');
    await upgradeFrpc(versionInfo.value.latest_version);
    upgradeProgress.value = isInstall ? t('settings.kernel.installSuccess') : t('settings.kernel.upgradeSuccess');
    await loadVersionInfo();
  } catch (e) {
    console.error("Failed to upgrade frpc:", e);
    upgradeProgress.value = isInstall ? t('settings.kernel.installFailed') : t('settings.kernel.upgradeFailed');
  } finally {
    isUpgrading.value = false;
  }
}

async function toggleServerEnable(server: ServerItem) {
  const updatedServer = { ...server, enable: !server.enable };
  try {
    await updateServer(updatedServer);
    const index = servers.value.findIndex((s) => s.id === server.id);
    if (index !== -1) {
      servers.value[index] = updatedServer;
    }
  } catch (e) {
    console.error("Failed to toggle server enable:", e);
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

        <section class="space-y-3">
          <header class="space-y-1">
            <h3 class="text-sm font-medium">{{ t('settings.homeDisplay.title') }}</h3>
            <p class="text-xs text-muted-foreground">{{ t('settings.homeDisplay.description') }}</p>
          </header>
          <div class="flex flex-wrap gap-2">
            <div
              v-if="servers.length === 0"
              class="text-sm text-muted-foreground py-2"
            >
              {{ t('settings.homeDisplay.empty') }}
            </div>
            <button
              v-for="server in servers"
              :key="server.id"
              class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium transition-all duration-200"
              :class="server.enable ? 'bg-primary text-primary-foreground shadow-sm' : 'bg-muted text-muted-foreground hover:bg-muted/80'"
              @click="toggleServerEnable(server)"
            >
              <span class="flex h-4 w-4 items-center justify-center text-[10px] font-bold">
                {{ server.id.toUpperCase() }}
              </span>
              {{ server.title }}
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
        <div v-if="versionInfo" class="rounded-xl border border-border bg-card p-5">
          <div class="flex items-center gap-3 mb-4">
            <h3 class="text-base font-semibold">frpc</h3>
            <span class="rounded bg-muted px-2 py-0.5 text-xs font-medium text-muted-foreground">{{ versionInfo.platform }}</span>
            <span class="rounded bg-muted px-2 py-0.5 text-xs font-medium text-muted-foreground">{{ versionInfo.arch }}</span>
            <span
              v-if="versionInfo.current_version === '0'"
              class="ml-auto inline-flex items-center text-yellow-500"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" x2="12" y1="8" y2="12"/>
                <line x1="12" x2="12.01" y1="16" y2="16"/>
              </svg>
            </span>
            <span
              v-else-if="versionInfo.can_upgrade"
              class="ml-auto inline-flex items-center gap-1 rounded-full bg-yellow-500/10 px-2.5 py-0.5 text-xs font-medium text-yellow-500"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" x2="12" y1="8" y2="12"/>
                <line x1="12" x2="12.01" y1="16" y2="16"/>
              </svg>
              {{ t('settings.kernel.updatable') }}
            </span>
            <span
              v-else
              class="ml-auto rounded-full bg-green-500/10 px-2.5 py-0.5 text-xs font-medium text-green-500"
            >
              {{ t('settings.kernel.latest') }}
            </span>
          </div>

          <div class="space-y-2 text-sm">
            <div class="flex items-center justify-between">
              <span class="text-muted-foreground">{{ t('settings.kernel.currentVersion') }}</span>
              <span v-if="versionInfo.current_version === '0'" class="font-medium text-muted-foreground">{{ t('settings.kernel.notInstalled') }}</span>
              <span v-else class="font-medium">{{ versionInfo.current_version }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-muted-foreground">{{ t('settings.kernel.latestVersion') }}</span>
              <span class="font-medium">{{ versionInfo.latest_version || '-' }}</span>
            </div>
          </div>

          <div v-if="upgradeProgress" class="mt-4 p-3 rounded-lg bg-muted/50 text-sm text-muted-foreground">
            {{ upgradeProgress }}
          </div>

          <div class="mt-4 flex justify-end">
            <button
              v-if="versionInfo.can_upgrade || versionInfo.current_version === '0'"
              :disabled="isUpgrading"
              class="inline-flex items-center gap-1.5 rounded-lg border border-border px-3 py-1.5 text-xs font-medium transition-colors hover:bg-muted/50 disabled:opacity-50 disabled:cursor-not-allowed"
              @click="handleUpgrade"
            >
              <svg v-if="!isUpgrading && versionInfo.current_version === '0'" xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="7 10 12 15 17 10"/>
                <line x1="12" x2="12" y1="15" y2="3"/>
              </svg>
              <svg v-else-if="!isUpgrading" xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                <path d="M3 3v5h5"/>
                <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
                <path d="M16 16h5v5"/>
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin">
                <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
              </svg>
              {{ isUpgrading ? (versionInfo.current_version === '0' ? t('settings.kernel.installing') : t('settings.kernel.upgrading')) : (versionInfo.current_version === '0' ? t('settings.kernel.install') : t('settings.kernel.update')) }}
            </button>
          </div>
        </div>

        <div v-else class="rounded-xl border border-border bg-card p-5">
          <div class="flex items-center justify-center h-20 text-sm text-muted-foreground">
            {{ t('settings.kernel.loading') }}
          </div>
        </div>
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
            <p class="text-xs text-muted-foreground">{{ t('settings.about.description') }}</p>
          </header>
        </section>

        <div class="rounded-xl border border-border bg-card p-5">
          <div class="flex items-center gap-4">
            <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-orange-500 to-amber-500 text-white">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"/>
                <path d="m7 10 3 3 7-7"/>
              </svg>
            </div>
            <div class="flex-1">
              <h3 class="text-base font-semibold">{{ t('settings.about.appName') }}</h3>
              <p class="text-xs text-muted-foreground">v0.1.0</p>
            </div>
            <div class="flex items-center gap-2">
              <a
                href="https://github.com/your-username/frpc-tray"
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center gap-1.5 rounded-lg border border-border px-3 py-1.5 text-xs font-medium transition-colors hover:bg-muted/50"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/>
                  <path d="M9 18c-4.51 2-5-2-7-2"/>
                </svg>
                GitHub
              </a>
              <button
                class="inline-flex items-center gap-1.5 rounded-lg border border-border px-3 py-1.5 text-xs font-medium transition-colors hover:bg-muted/50"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/>
                  <polyline points="14 2 14 8 20 8"/>
                  <line x1="16" x2="8" y1="13" y2="13"/>
                  <line x1="16" x2="8" y1="17" y2="17"/>
                  <line x1="10" x2="8" y1="9" y2="9"/>
                </svg>
                {{ t('settings.about.changelog') }}
              </button>
              <button
                class="inline-flex items-center gap-1.5 rounded-lg bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground transition-colors hover:bg-primary/90"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                  <path d="M3 3v5h5"/>
                  <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
                  <path d="M16 16h5v5"/>
                </svg>
                {{ t('settings.about.checkUpdate') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <ConfirmDialog
      :show="showDeleteDialog"
      :title="t('common.confirm')"
      :message="t('server.deleteConfirm')"
      @confirm="confirmDelete"
      @cancel="cancelDelete"
    />
  </div>
</template>
