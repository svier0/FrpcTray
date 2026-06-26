<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from "radix-vue";
import { useI18n } from "vue-i18n";

export interface ProxyItem {
  id: string;
  name: string;
  desc?: string;
  url: string;
  enabled: boolean;
  icon?: string;
  color?: string;
}

const props = defineProps<{
  item: ProxyItem;
  isActive?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:enabled", id: string, value: boolean): void;
  (e: "edit", id: string): void;
  (e: "duplicate", id: string): void;
  (e: "delete", id: string): void;
  (e: "viewLogs", id: string): void;
}>();

const { t } = useI18n();
</script>

<template>
  <div
    class="group relative overflow-hidden rounded-xl border border-border p-4 transition-all duration-300 bg-card text-card-foreground hover:border-border hover:shadow-sm"
    :class="{
      'border-blue-500/60 shadow-sm shadow-blue-500/10': props.isActive,
    }"
  >
    <div
      class="absolute inset-0 bg-gradient-to-r to-transparent transition-opacity duration-500 pointer-events-none"
      :class="props.isActive ? 'from-blue-500/10 opacity-100' : 'from-primary/10 opacity-0'"
    />

    <div class="relative flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div class="flex min-w-0 flex-1 items-center gap-3">
        <button
          class="drag-handle -ml-1.5 flex-shrink-0 cursor-grab active:cursor-grabbing p-1.5 text-muted-foreground/50 hover:text-muted-foreground transition-colors"
          :title="t('proxy.dragHandle')"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="9" cy="5" r="1" />
            <circle cx="9" cy="12" r="1" />
            <circle cx="9" cy="19" r="1" />
            <circle cx="15" cy="5" r="1" />
            <circle cx="15" cy="12" r="1" />
            <circle cx="15" cy="19" r="1" />
          </svg>
        </button>

        <div
          class="h-8 w-8 flex-shrink-0 rounded-lg bg-muted flex items-center justify-center border border-border group-hover:scale-105 transition-transform duration-300"
        >
          <div
            class="w-5 h-5 rounded flex items-center justify-center text-xs font-semibold"
            :style="props.item.color ? { color: props.item.color } : {}"
          >
            {{ props.item.name.charAt(0).toUpperCase() }}
          </div>
        </div>

        <div class="min-w-0 flex-1 space-y-1">
          <div class="flex items-center gap-2 min-h-6">
            <h3 class="text-sm font-semibold leading-none truncate">
              {{ props.item.desc || props.item.name }}
            </h3>
          </div>

          <p class="text-xs text-muted-foreground truncate">
            {{ props.item.name }}
          </p>
        </div>
      </div>

      <div class="flex items-center ml-auto min-w-0 gap-2">
        <SwitchRoot
          :checked="props.item.enabled"
          class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-blue-500 data-[state=unchecked]:bg-muted"
          @update:checked="(val: boolean) => emit('update:enabled', props.item.id, val)"
        >
          <SwitchThumb
            class="pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0"
          />
        </SwitchRoot>

        <div class="flex items-center gap-1 opacity-0 pointer-events-none group-hover:opacity-100 group-focus-within:opacity-100 group-hover:pointer-events-auto group-focus-within:pointer-events-auto transition-opacity duration-200">
          <button
            class="inline-flex items-center justify-center h-7 w-7 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
            :title="t('proxy.viewLogs')"
            @click="emit('viewLogs', props.item.id)"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="16" y1="13" x2="8" y2="13" />
              <line x1="16" y1="17" x2="8" y2="17" />
              <polyline points="10 9 9 9 8 9" />
            </svg>
          </button>

          <button
            class="inline-flex items-center justify-center h-7 w-7 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
            :title="t('common.edit')"
            @click="emit('edit', props.item.id)"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
              <path d="m15 5 4 4" />
            </svg>
          </button>

          <button
            class="inline-flex items-center justify-center h-7 w-7 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
            :title="t('common.duplicate')"
            @click="emit('duplicate', props.item.id)"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
              <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
            </svg>
          </button>

          <button
            class="inline-flex items-center justify-center h-7 w-7 rounded-lg text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors"
            :title="t('common.delete')"
            @click="emit('delete', props.item.id)"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M3 6h18" />
              <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
              <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
              <line x1="10" y1="11" x2="10" y2="17" />
              <line x1="14" y1="11" x2="14" y2="17" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
