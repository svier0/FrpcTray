<script setup lang="ts">
import { VueDraggable } from "vue-draggable-plus";
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
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

watch(
  () => props.items,
  (newItems) => {
    dragList.value = [...newItems];
  },
  { deep: true }
);

function onUpdate() {
  emit("update:items", [...dragList.value]);
}
</script>

<template>
  <VueDraggable
    v-model="dragList"
    :animation="150"
    handle=".drag-handle"
    ghost-class="opacity-50"
    @update="onUpdate"
  >
    <div v-for="item in dragList" :key="item.id" class="mb-3">
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
  </VueDraggable>

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
