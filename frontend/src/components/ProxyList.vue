<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from "vue";
import { useI18n } from "vue-i18n";
import Sortable from "sortablejs";
import ProxyItem from "./ProxyItem.vue";
import type { ProxyItem as ProxyItemType } from "./ProxyItem.vue";

const props = defineProps<{
  items: ProxyItemType[];
  activeId?: string;
}>();

const emit = defineEmits<{
  (e: "update:items", value: ProxyItemType[]): void;
  (e: "update:enabled", id: string, value: boolean): void;
  (e: "edit", id: string): void;
  (e: "duplicate", id: string): void;
  (e: "delete", id: string): void;
  (e: "viewLogs", id: string): void;
}>();

const { t } = useI18n();

const dragList = ref<ProxyItemType[]>([...props.items]);
const listRef = ref<HTMLElement | null>(null);
let sortable: Sortable | null = null;

watch(
  () => props.items,
  (newItems) => {
    dragList.value = [...newItems];
  },
  { deep: true }
);

function initSortable() {
  if (listRef.value) {
    sortable = Sortable.create(listRef.value, {
      animation: 150,
      handle: ".drag-handle",
      ghostClass: "opacity-50",
      chosenClass: "sortable-chosen",
      forceFallback: true,
      fallbackClass: "sortable-fallback",
      onEnd: (evt) => {
        const oldIndex = evt.oldIndex;
        const newIndex = evt.newIndex;
        if (oldIndex !== undefined && newIndex !== undefined && oldIndex !== newIndex) {
          const item = dragList.value.splice(oldIndex, 1)[0];
          dragList.value.splice(newIndex, 0, item);
        }
        emit("update:items", [...dragList.value]);
      },
    });
  }
}

onMounted(() => {
  nextTick(initSortable);
});

onBeforeUnmount(() => {
  sortable?.destroy();
  sortable = null;
});
</script>

<template>
  <div ref="listRef" class="space-y-3 pt-4">
    <div v-for="item in dragList" :key="item.id" class="sortable-item">
      <ProxyItem
        :item="item"
        :is-active="item.id === activeId"
        @update:enabled="(id, val) => emit('update:enabled', id, val)"
        @edit="(id) => emit('edit', id)"
        @duplicate="(id) => emit('duplicate', id)"
        @delete="(id) => emit('delete', id)"
        @view-logs="(id) => emit('viewLogs', id)"
      />
    </div>
  </div>

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
      <rect width="18" height="18" x="3" y="3" rx="2" />
      <path d="M3 9h18" />
      <path d="M9 21V9" />
    </svg>
    <p class="text-sm">{{ t('proxy.empty') }}</p>
    <p class="text-xs mt-1">{{ t('proxy.emptyHint') }}</p>
  </div>
</template>

<style scoped>
.sortable-item {
  cursor: default;
}
.sortable-item:global(.sortable-chosen) {
  opacity: 0.5;
}
.sortable-item:global(.sortable-fallback) {
  opacity: 0.8;
}
</style>
