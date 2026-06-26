<script setup lang="ts">
import { VueDraggable } from "vue-draggable-plus";
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import ServerItem from "./ServerItem.vue";
import type { ServerItem as ServerItemType } from "./ServerItem.vue";

const props = defineProps<{
  items: ServerItemType[];
}>();

const emit = defineEmits<{
  (e: "update:items", value: ServerItemType[]): void;
  (e: "save", id: string, data: ServerItemType): void;
  (e: "delete", id: string): void;
  (e: "add"): void;
}>();

const { t } = useI18n();

const dragList = ref<ServerItemType[]>([...props.items]);
const expandedId = ref<string | null>(null);

watch(
  () => props.items,
  (newItems) => {
    dragList.value = [...newItems];
  },
  { deep: true }
);

function onEnd() {
  emit("update:items", [...dragList.value]);
}

function handleToggleExpand(id: string) {
  expandedId.value = expandedId.value === id ? null : id;
}

function handleSave(id: string, data: ServerItemType) {
  emit("save", id, data);
}

function handleDelete(id: string) {
  emit("delete", id);
}
</script>

<template>
  <div class="space-y-3">
    <VueDraggable
      v-model="dragList"
      :animation="150"
      handle=".drag-handle"
      @end="onEnd"
    >
      <div v-for="item in dragList" :key="item.id" class="relative group">
        <button
          class="drag-handle absolute -left-1 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 p-1 text-muted-foreground/50 hover:text-muted-foreground transition-all cursor-grab active:cursor-grabbing"
          :title="t('proxy.dragHandle')"
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
            <circle cx="9" cy="5" r="1" />
            <circle cx="9" cy="12" r="1" />
            <circle cx="9" cy="19" r="1" />
            <circle cx="15" cy="5" r="1" />
            <circle cx="15" cy="12" r="1" />
            <circle cx="15" cy="19" r="1" />
          </svg>
        </button>

        <ServerItem
          :item="item"
          :is-expanded="expandedId === item.id"
          @toggle-expand="handleToggleExpand"
          @save="handleSave"
          @delete="handleDelete"
        />
      </div>
    </VueDraggable>

    <button
      class="flex w-full items-center justify-center gap-2 rounded-xl border border-dashed border-border p-4 text-sm text-muted-foreground transition-colors hover:border-blue-500/50 hover:bg-blue-500/5 hover:text-blue-500"
      @click="emit('add')"
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
        <line x1="12" x2="12" y1="5" y2="19" />
        <line x1="5" x2="19" y1="12" y2="12" />
      </svg>
      {{ t('server.add') }}
    </button>

    <div
      v-if="dragList.length === 0"
      class="flex flex-col items-center justify-center py-12 text-muted-foreground"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="48"
        height="48"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="mb-4 opacity-50"
      >
        <rect width="20" height="8" x="2" y="2" rx="2" ry="2" />
        <rect width="20" height="8" x="2" y="14" rx="2" ry="2" />
        <line x1="6" x2="6.01" y1="6" y2="6" />
        <line x1="6" x2="6.01" y1="18" y2="18" />
      </svg>
      <p class="text-sm">{{ t('server.empty') }}</p>
      <p class="text-xs mt-1">{{ t('server.emptyHint') }}</p>
    </div>
  </div>
</template>
