<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { ServerItem } from "./ServerItem.vue";

type ServerStatus = "idle" | "connecting" | "running" | "error";

const props = defineProps<{
  activeTab: string;
  enabledServers: ServerItem[];
  serverStatus: Record<string, ServerStatus>;
  serverError: Record<string, string>;
  anyServerRunning: boolean;
  anyServerConnecting: boolean;
  isTogglingAll: boolean;
}>();

const emit = defineEmits<{
  (e: "update:activeTab", value: string): void;
  (e: "openSettings"): void;
  (e: "addProxy"): void;
  (e: "toggleAll"): void;
}>();

const { t } = useI18n();

function setActiveTab(tab: string) {
  emit("update:activeTab", tab);
}

function getStatusColor(id: string): string {
  if (props.serverError[id]) return "bg-red-500";
  const status = props.serverStatus[id] || "idle";
  if (status === "connecting") return "bg-blue-500";
  if (status === "running") return "bg-emerald-500";
  if (status === "error") return "bg-red-500";
  return "bg-muted-foreground/30";
}
</script>

<template>
  <header
    class="fixed top-0 left-0 right-0 z-50 h-14 flex items-center justify-between px-4 border-b border-border bg-background/80 backdrop-blur-md"
    data-tauri-drag-region
  >
    <div
      class="flex items-center gap-2"
      style="WebkitAppRegion: no-drag"
    >
      <span class="text-lg font-semibold text-orange-500">FrpC Tray</span>

      <button
        class="inline-flex items-center justify-center h-8 w-8 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        :title="t('header.settings')"
        @click="emit('openSettings')"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
          <circle cx="12" cy="12" r="3" />
        </svg>
      </button>

      <button
        class="ml-1 inline-flex items-center justify-center h-7 px-2.5 rounded-lg text-xs font-medium transition-colors"
        :class="props.isTogglingAll ? 'bg-muted text-muted-foreground cursor-not-allowed' : props.anyServerRunning ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20' : 'bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/20'"
        :disabled="props.isTogglingAll"
        @click="emit('toggleAll')"
      >
        <template v-if="props.isTogglingAll">操作中…</template>
        <template v-else-if="props.anyServerRunning">全部停止</template>
        <template v-else>全部启动</template>
      </button>
    </div>

    <div
      class="flex items-center gap-2"
      style="WebkitAppRegion: no-drag"
    >
      <div class="flex items-center gap-1 p-1 bg-muted rounded-xl">
        <button
          v-for="server in props.enabledServers"
          :key="server.id"
          class="relative inline-flex items-center justify-center h-7 px-2 rounded-lg transition-colors text-xs font-medium truncate max-w-[80px]"
          :class="props.activeTab === server.id ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'"
          @click="setActiveTab(server.id)"
        >
          <span
            class="absolute -top-0.5 -right-0.5 h-2 w-2 rounded-full"
            :class="getStatusColor(server.id)"
          />
          {{ server.title }}
        </button>
      </div>

      <button
        class="inline-flex items-center justify-center h-8 w-8 rounded-full bg-orange-500 text-white shadow-lg shadow-orange-500/30 hover:bg-orange-600 transition-colors"
        :title="t('header.addProxy')"
        @click="emit('addProxy')"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M5 12h14" />
          <path d="M12 5v14" />
        </svg>
      </button>
    </div>
  </header>
</template>
