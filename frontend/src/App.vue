<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import AppHeader from "./components/AppHeader.vue";
import ProxyList from "./components/ProxyList.vue";
import type { ProxyItem } from "./components/ProxyItem.vue";
import SettingsPage from "./components/SettingsPage.vue";
import type { ServerItem } from "./components/ServerItem.vue";
import { listServers, listProxies } from "./utils/ipc";

const globalEnabled = ref(true);
const activeTab = ref<string>("");
const activeProxyId = ref<string | undefined>();
const showSettings = ref(false);
const servers = ref<ServerItem[]>([]);
const proxies = ref<ProxyItem[]>([]);
const enabledServers = ref<ServerItem[]>([]);
const isLoadingProxies = ref(false);

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
      url: p.customDomains?.[0] || `${p.type}://localhost:${p.localPort}`,
      enabled: true,
    }));
  } catch (e) {
    console.error("Failed to load proxies:", e);
    proxies.value = [];
  } finally {
    isLoadingProxies.value = false;
  }
}

function handleUpdateItems(newItems: ProxyItem[]) {
  proxies.value = newItems;
}

function handleUpdateEnabled(id: string, value: boolean) {
  const proxy = proxies.value.find((p) => p.id === id);
  if (proxy) {
    proxy.enabled = value;
  }
}

function handleEdit(id: string) {
  console.log("edit", id);
}

function handleDuplicate(id: string) {
  const source = proxies.value.find((p) => p.id === id);
  if (source) {
    const newProxy: ProxyItem = {
      id: String(Date.now()),
      name: `${source.name} copy`,
      url: source.url,
      enabled: false,
    };
    proxies.value.push(newProxy);
  }
}

function handleDelete(id: string) {
  proxies.value = proxies.value.filter((p) => p.id !== id);
}

function handleViewLogs(id: string) {
  console.log("viewLogs", id);
}

function handleAddProxy() {
  console.log("addProxy");
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
  </div>
</template>
