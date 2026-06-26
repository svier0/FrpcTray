<script setup lang="ts">
import { ref, onMounted } from "vue";
import AppHeader from "./components/AppHeader.vue";
import ProxyList from "./components/ProxyList.vue";
import type { ProxyItem } from "./components/ProxyItem.vue";
import SettingsPage from "./components/SettingsPage.vue";
import type { ServerItem } from "./components/ServerItem.vue";
import { listServers } from "./utils/ipc";

const globalEnabled = ref(true);
const activeTab = ref<string>("proxy");
const activeProxyId = ref<string | undefined>();
const showSettings = ref(false);
const servers = ref<ServerItem[]>([]);

const proxies = ref<ProxyItem[]>([
  {
    id: "1",
    name: "Claude Desktop Official",
    url: "http://claude.ai/download",
    enabled: true,
  },
  {
    id: "2",
    name: "j7yx",
    url: "http://newapi.x.j7yx.com/v1",
    enabled: false,
  },
]);

const enabledServers = ref<ServerItem[]>([]);

async function loadServers() {
  try {
    servers.value = await listServers();
    enabledServers.value = servers.value.filter((s) => s.enable);
  } catch (e) {
    console.error("Failed to load servers:", e);
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
  const newProxy: ProxyItem = {
    id: String(Date.now()),
    name: "New Proxy",
    url: "http://localhost:8080",
    enabled: false,
  };
  proxies.value.push(newProxy);
}

function handleOpenSettings() {
  showSettings.value = true;
}

function handleCloseSettings() {
  showSettings.value = false;
  loadServers();
}

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
        <ProxyList
          v-if="activeTab === 'proxy'"
          :items="proxies"
          :active-id="activeProxyId"
          @update:items="handleUpdateItems"
          @update:enabled="handleUpdateEnabled"
          @edit="handleEdit"
          @duplicate="handleDuplicate"
          @delete="handleDelete"
          @view-logs="handleViewLogs"
        />

        <div v-else class="flex flex-col items-center justify-center h-full text-muted-foreground">
          <p class="text-sm">{{ activeTab }} 视图</p>
        </div>
      </main>
    </template>

    <SettingsPage
      v-else
      @close="handleCloseSettings"
    />
  </div>
</template>
