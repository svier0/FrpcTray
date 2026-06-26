<script setup lang="ts">
import { useI18n } from "vue-i18n";

const props = defineProps<{
  show: boolean;
  title: string;
  message: string;
}>();

const emit = defineEmits<{
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();

const { t } = useI18n();
</script>

<template>
  <Teleport to="body">
    <div
      v-if="show"
      class="fixed inset-0 z-50 flex items-center justify-center"
    >
      <div
        class="fixed inset-0 bg-black/50 backdrop-blur-sm"
        @click="emit('cancel')"
      />
      <div class="relative z-10 w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-lg">
        <h3 class="text-base font-semibold mb-2">{{ title }}</h3>
        <p class="text-sm text-muted-foreground mb-6">{{ message }}</p>
        <div class="flex justify-end gap-2">
          <button
            class="inline-flex items-center justify-center rounded-lg border border-border bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-muted"
            @click="emit('cancel')"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            class="inline-flex items-center justify-center rounded-lg bg-destructive px-4 py-2 text-sm font-medium text-destructive-foreground transition-colors hover:bg-destructive/90"
            @click="emit('confirm')"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
