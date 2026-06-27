<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import AppHeader from "./components/AppHeader.vue";
import ProxyList from "./components/ProxyList.vue";
import type { ProxyItem } from "./components/ProxyItem.vue";
import ProxyDialog from "./components/ProxyDialog.vue";
import type { ProxyFormData } from "./components/ProxyDialog.vue";
import SettingsPage from "./components/SettingsPage.vue";
import type { ServerItem } from "./components/ServerItem.vue";
import { listServers, listProxies, createProxy, updateProxy, deleteProxy, reorderProxies } from "./utils/ipc";

const globalEnabled = ref(false);
const activeTab = ref<string>("");
const activeProxyId = ref<string | undefined>();
const showSettings = ref(false);
const servers = ref<ServerItem[]>([]);
const proxies = ref<ProxyItem[]>([]);
const enabledServers = ref<ServerItem[]>([]);
const isLoadingProxies = ref(false);

// Mock 服务器运行状态: idle / running / error
type ServerStatus = "idle" | "running" | "error";
const serverStatus = ref<Record<string, ServerStatus>>({});

const showProxyDialog = ref(false);
const proxyDialogMode = ref<"create" | "edit">("create");
const editingProxy = ref<ProxyFormData | undefined>();
const editingProxyOldName = ref("");

async function loadServers() {
  try {
    servers.value = await listServers();
    enabledServers.value = servers.value.filter((s) => s.enable);
    if (enabledServers.value.length > 0 && !activeTab.value) {
      activeTab.value = enabledServers.value[0].id;
    }
  } catch (e) {
    console.error("Failed to load servers:", e);
  }
}

async function loadProxies(serverId: string) {
  if (!serverId) {
    proxies.value = [];
    return;
  }
  try {
    isLoadingProxies.value = true;
    const rawProxies = await listProxies(serverId);
    proxies.value = rawProxies.map((p, index) => ({
      id: String(index),
      name: p.name,
      desc: p.desc || undefined,
      type: p.type,
      enabled: p.enabled,
      localIP: p.localIP || undefined,
      localPort: p.localPort,
      remotePort: p.remotePort || undefined,
      customDomains: p.customDomains || undefined,
      locations: p.locations || undefined,
    }));
  } catch (e) {
    console.error("Failed to load proxies:", e);
    proxies.value = [];
  } finally {
    isLoadingProxies.value = false;
  }
}

async function handleUpdateItems(newItems: ProxyItem[]) {
  proxies.value = newItems;
  if (activeTab.value) {
    try {
      await reorderProxies(activeTab.value, newItems.map((p) => p.name));
    } catch (e) {
      console.error("Failed to reorder proxies:", e);
    }
  }
}

async function handleUpdateEnabled(id: string, value: boolean) {
  const proxy = proxies.value.find((p) => p.id === id);
  if (proxy && activeTab.value) {
    proxy.enabled = value;
    try {
      await updateProxy(activeTab.value, proxy.name, {
        name: proxy.name,
        desc: proxy.desc,
        enabled: value,
        type: proxy.type || "tcp",
        localIP: proxy.localIP,
        localPort: proxy.localPort || 0,
        remotePort: proxy.remotePort,
        customDomains: proxy.customDomains,
        locations: proxy.locations,
      });
    } catch (e) {
      console.error("Failed to update proxy enabled:", e);
      proxy.enabled = !value;
    }
  }
}

function handleEdit(id: string) {
  const proxy = proxies.value.find((p) => p.id === id);
  if (proxy && activeTab.value) {
    editingProxyOldName.value = proxy.name;
    editingProxy.value = {
      name: proxy.name,
      desc: proxy.desc || "",
      type: proxy.type || "tcp",
      localIP: proxy.localIP || "127.0.0.1",
      localPort: proxy.localPort || 8080,
      remotePort: proxy.remotePort || null,
      customDomains: proxy.customDomains ? proxy.customDomains.join(", ") : "",
      locations: proxy.locations ? proxy.locations.join(", ") : "",
    };
    proxyDialogMode.value = "edit";
    showProxyDialog.value = true;
  }
}

async function handleDuplicate(id: string) {
  const source = proxies.value.find((p) => p.id === id);
  if (source && activeTab.value) {
    const newName = `${source.name}_copy`;
    try {
      await createProxy(activeTab.value, {
        name: newName,
        desc: source.desc,
        enabled: false,
        type: source.type || "tcp",
        localIP: source.localIP,
        localPort: source.localPort || 0,
        remotePort: source.remotePort,
        customDomains: source.customDomains,
        locations: source.locations,
      });
      await loadProxies(activeTab.value);
    } catch (e) {
      console.error("Failed to duplicate proxy:", e);
    }
  }
}

async function handleDelete(id: string) {
  const proxy = proxies.value.find((p) => p.id === id);
  if (proxy && activeTab.value) {
    try {
      await deleteProxy(activeTab.value, proxy.name);
      await loadProxies(activeTab.value);
    } catch (e) {
      console.error("Failed to delete proxy:", e);
    }
  }
}

function handleViewLogs(id: string) {
  console.log("viewLogs", id);
}

function handleAddProxy() {
  editingProxy.value = undefined;
  proxyDialogMode.value = "create";
  showProxyDialog.value = true;
}

async function handleProxySubmit(data: ProxyFormData) {
  if (!activeTab.value) return;

  const proxyData = {
    name: data.name,
    desc: data.desc || undefined,
    enabled: false,
    type: data.type,
    localIP: data.localIP || undefined,
    localPort: data.localPort,
    remotePort: data.remotePort || undefined,
    customDomains: data.customDomains ? data.customDomains.split(",").map((s) => s.trim()).filter(Boolean) : undefined,
    locations: data.locations ? data.locations.split(",").map((s) => s.trim()).filter(Boolean) : undefined,
  };

  try {
    if (proxyDialogMode.value === "create") {
      await createProxy(activeTab.value, proxyData);
    } else {
      await updateProxy(activeTab.value, editingProxyOldName.value, proxyData);
    }
    await loadProxies(activeTab.value);
  } catch (e) {
    console.error("Failed to save proxy:", e);
  }
}

function handleOpenSettings() {
  showSettings.value = true;
}

function handleCloseSettings() {
  showSettings.value = false;
  loadServers();
}

function getActiveServerStatus(): ServerStatus {
  return serverStatus.value[activeTab.value] || "idle";
}

function toggleServerRun() {
  const id = activeTab.value;
  if (!id) return;
  const current = serverStatus.value[id] || "idle";
  serverStatus.value[id] = current === "running" ? "idle" : "running";
}

function handleViewServerLogs() {
  console.log("view server logs", activeTab.value);
}

watch(activeTab, (newTab) => {
  if (newTab) {
    loadProxies(newTab);
  }
});

onMounted(() => {
  loadServers();
});
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden bg-background text-foreground">
    <template v-if="!showSettings">
      <AppHeader
        v-model:global-enabled="globalEnabled"
        v-model:active-tab="activeTab"
        :enabled-servers="enabledServers"
        :server-status="serverStatus"
        @open-settings="handleOpenSettings"
        @add-proxy="handleAddProxy"
      />

      <main class="flex-1 overflow-y-auto pt-14 px-4 pb-4">
        <div v-if="activeTab" class="mb-4 p-3 rounded-xl border border-border bg-card">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <div
                class="h-2.5 w-2.5 rounded-full"
                :class="{
                  'bg-blue-500': getActiveServerStatus() === 'running',
                  'bg-red-500': getActiveServerStatus() === 'error',
                  'bg-muted-foreground/30': getActiveServerStatus() === 'idle'
                }"
              />
              <span class="text-sm font-medium">
                {{ getActiveServerStatus() === 'running' ? '运行中' : getActiveServerStatus() === 'error' ? '异常' : '已停止' }}
              </span>
            </div>
            <div class="flex items-center gap-2">
              <button
                class="inline-flex items-center gap-1.5 h-7 px-2.5 rounded-lg text-xs font-medium text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
                @click="handleViewServerLogs"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                  <polyline points="14 2 14 8 20 8"/>
                </svg>
                日志
              </button>
              <button
                class="inline-flex items-center gap-1.5 h-7 px-2.5 rounded-lg text-xs font-medium transition-colors"
                :class="getActiveServerStatus() === 'running' ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20' : 'bg-blue-500 text-white hover:bg-blue-600'"
                @click="toggleServerRun"
              >
                <svg v-if="getActiveServerStatus() !== 'running'" xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polygon points="5 3 19 12 5 21 5 3"/>
                </svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect width="4" height="16" x="6" y="4"/>
                  <rect width="4" height="16" x="14" y="4"/>
                </svg>
                {{ getActiveServerStatus() === 'running' ? '停止' : '启动' }}
              </button>
            </div>
          </div>
          <div v-if="getActiveServerStatus() === 'error'" class="mt-2 p-2 rounded-lg bg-red-500/10 text-red-500 text-xs">
            连接服务器失败，请检查配置或查看日志
          </div>
        </div>

        <div v-if="isLoadingProxies" class="flex items-center justify-center h-32 text-muted-foreground">
          <p class="text-sm">加载中...</p>
        </div>
        <ProxyList
          v-else
          :items="proxies"
          :active-id="activeProxyId"
          @update:items="handleUpdateItems"
          @update:enabled="handleUpdateEnabled"
          @edit="handleEdit"
          @duplicate="handleDuplicate"
          @delete="handleDelete"
          @view-logs="handleViewLogs"
        />
      </main>
    </template>

    <SettingsPage
      v-else
      @close="handleCloseSettings"
    />

    <ProxyDialog
      :open="showProxyDialog"
      :mode="proxyDialogMode"
      :initial-data="editingProxy"
      @update:open="showProxyDialog = $event"
      @submit="handleProxySubmit"
    />
  </div>
</template>
