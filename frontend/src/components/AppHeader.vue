<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from "radix-vue";
import { useI18n } from "vue-i18n";
import type { ServerItem } from "./ServerItem.vue";

type ServerStatus = "idle" | "connecting" | "running" | "error";

const props = defineProps<{
  globalEnabled: boolean;
  activeTab: string;
  enabledServers: ServerItem[];
  serverStatus: Record<string, ServerStatus>;
}>();

const emit = defineEmits<{
  (e: "update:globalEnabled", value: boolean): void;
  (e: "update:activeTab", value: string): void;
  (e: "openSettings"): void;
  (e: "addProxy"): void;
}>();

const { t } = useI18n();

function setActiveTab(tab: string) {
  emit("update:activeTab", tab);
}

function getStatusColor(id: string): string {
  const status = props.serverStatus[id] || "idle";
  if (status === "running") return "bg-blue-500";
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

      <SwitchRoot
        :checked="props.globalEnabled"
        class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-muted ml-1"
        @update:checked="(val: boolean) => emit('update:globalEnabled', val)"
      >
        <SwitchThumb
          class="pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0"
        />
      </SwitchRoot>
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
