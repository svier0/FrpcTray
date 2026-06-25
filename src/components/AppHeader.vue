<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from "radix-vue";

export type ViewTab = "proxy" | "terminal" | "tools" | "mobile" | "history" | "validate";

const props = defineProps<{
  globalEnabled: boolean;
  activeTab: ViewTab;
}>();

const emit = defineEmits<{
  (e: "update:globalEnabled", value: boolean): void;
  (e: "update:activeTab", value: ViewTab): void;
  (e: "openSettings"): void;
  (e: "addProxy"): void;
}>();

function setActiveTab(tab: ViewTab) {
  emit("update:activeTab", tab);
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
      <span class="text-lg font-semibold text-orange-500">frpc tray</span>

      <button
        class="inline-flex items-center justify-center h-8 w-8 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        title="设置"
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
      <div class="flex items-center gap-0.5 p-1 bg-muted rounded-xl">
        <button
          class="inline-flex items-center justify-center h-7 w-7 rounded-lg transition-colors"
          :class="props.activeTab === 'proxy' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'"
          title="代理"
          @click="setActiveTab('proxy')"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10" />
            <line x1="2" y1="12" x2="22" y2="12" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
          </svg>
        </button>
        <button
          class="inline-flex items-center justify-center h-7 w-7 rounded-lg transition-colors"
          :class="props.activeTab === 'terminal' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'"
          title="终端"
          @click="setActiveTab('terminal')"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="4 17 10 11 4 5" />
            <line x1="12" y1="19" x2="20" y2="19" />
          </svg>
        </button>
      </div>

      <div class="flex items-center gap-0.5 p-1 bg-muted rounded-xl">
        <button
          class="inline-flex items-center justify-center h-7 w-7 rounded-lg transition-colors"
          :class="props.activeTab === 'tools' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'"
          title="工具"
          @click="setActiveTab('tools')"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
          </svg>
        </button>
        <button
          class="inline-flex items-center justify-center h-7 w-7 rounded-lg transition-colors"
          :class="props.activeTab === 'mobile' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'"
          title="移动设备"
          @click="setActiveTab('mobile')"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect width="14" height="20" x="5" y="2" rx="2" ry="2" />
            <path d="M12 18h.01" />
          </svg>
        </button>
        <button
          class="inline-flex items-center justify-center h-7 w-7 rounded-lg transition-colors"
          :class="props.activeTab === 'history' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'"
          title="历史"
          @click="setActiveTab('history')"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
            <path d="M3 3v5h5" />
            <path d="M12 7v5l4 2" />
          </svg>
        </button>
        <button
          class="inline-flex items-center justify-center h-7 w-7 rounded-lg transition-colors"
          :class="props.activeTab === 'validate' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'"
          title="验证"
          @click="setActiveTab('validate')"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </button>
      </div>

      <button
        class="inline-flex items-center justify-center h-8 w-8 rounded-full bg-orange-500 text-white shadow-lg shadow-orange-500/30 hover:bg-orange-600 transition-colors"
        title="添加代理"
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
