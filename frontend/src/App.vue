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

const globalEnabled = ref(true);
const activeTab = ref<string>("");
const activeProxyId = ref<string | undefined>();
const showSettings = ref(false);
const servers = ref<ServerItem[]>([]);
const proxies = ref<ProxyItem[]>([]);
const enabledServers = ref<ServerItem[]>([]);
const isLoadingProxies = ref(false);

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

function handleDuplicate(id: string) {
  const source = proxies.value.find((p) => p.id === id);
  if (source) {
    editingProxy.value = {
      name: `${source.name} copy`,
      desc: source.desc || "",
      type: source.type || "tcp",
      localIP: source.localIP || "127.0.0.1",
      localPort: source.localPort || 8080,
      remotePort: source.remotePort || null,
      customDomains: source.customDomains ? source.customDomains.join(", ") : "",
      locations: source.locations ? source.locations.join(", ") : "",
    };
    proxyDialogMode.value = "create";
    showProxyDialog.value = true;
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
        @open-settings="handleOpenSettings"
        @add-proxy="handleAddProxy"
      />

      <main class="flex-1 overflow-y-auto pt-14 px-4 pb-4">
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
