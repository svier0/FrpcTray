<script setup lang="ts">
import { ref } from "vue";
import AppHeader from "./components/AppHeader.vue";
import ProxyList from "./components/ProxyList.vue";
import type { ProxyItem } from "./components/ProxyItem.vue";

const globalEnabled = ref(true);
const activeProxyId = ref<string | undefined>();

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

function handleDragStart(id: string) {
  console.log("dragStart", id);
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
  console.log("openSettings");
}

function handleRefresh() {
  console.log("refresh");
}
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden bg-background text-foreground">
    <AppHeader
      v-model:global-enabled="globalEnabled"
      @open-settings="handleOpenSettings"
      @add-proxy="handleAddProxy"
      @refresh="handleRefresh"
    />

    <main class="flex-1 overflow-y-auto pt-14 px-4 pb-4">
      <ProxyList
        :items="proxies"
        :active-id="activeProxyId"
        @update:enabled="handleUpdateEnabled"
        @edit="handleEdit"
        @duplicate="handleDuplicate"
        @delete="handleDelete"
        @view-logs="handleViewLogs"
        @drag-start="handleDragStart"
      />
    </main>
  </div>
</template>
