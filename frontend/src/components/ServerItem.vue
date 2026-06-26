<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";

export interface ServerItem {
  id: string;
  title: string;
  enable: boolean;
  sort: number;
  serverAddr: string;
  serverPort: number;
  auth?: {
    method?: string;
    token?: string;
  } | null;
}

const props = defineProps<{
  item: ServerItem;
  isExpanded?: boolean;
}>();

const emit = defineEmits<{
  (e: "toggle-expand", id: string): void;
  (e: "save", id: string, data: ServerItem): void;
  (e: "delete", id: string): void;
}>();

const { t } = useI18n();

const editData = ref({
  title: props.item.title,
  serverAddr: props.item.serverAddr,
  serverPort: props.item.serverPort,
  authMethod: props.item.auth?.method || "",
  authToken: props.item.auth?.token || "",
});

const showAuthToken = ref(false);

function handleSave() {
  const data: ServerItem = {
    ...props.item,
    title: editData.value.title,
    serverAddr: editData.value.serverAddr,
    serverPort: editData.value.serverPort,
    auth: {
      method: editData.value.authMethod || undefined,
      token: editData.value.authToken || undefined,
    },
  };
  emit("save", props.item.id, data);
}

function handleDelete() {
  emit("delete", props.item.id);
}
</script>

<template>
  <div
    class="rounded-xl border border-border bg-card text-card-foreground transition-all duration-300"
    :class="{
      'border-blue-500/60 shadow-sm shadow-blue-500/10': isExpanded,
    }"
  >
    <button
      class="flex w-full items-center gap-3 p-4 text-left transition-colors hover:bg-muted/50"
      @click="emit('toggle-expand', props.item.id)"
    >
      <div
        class="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg border border-border bg-muted"
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
          class="text-muted-foreground"
        >
          <rect width="20" height="8" x="2" y="2" rx="2" ry="2" />
          <rect width="20" height="8" x="2" y="14" rx="2" ry="2" />
          <line x1="6" x2="6.01" y1="6" y2="6" />
          <line x1="6" x2="6.01" y1="18" y2="18" />
        </svg>
      </div>

      <div class="min-w-0 flex-1">
        <h3 class="text-sm font-semibold truncate">{{ props.item.title }}</h3>
        <p class="text-xs text-muted-foreground truncate">
          {{ props.item.serverAddr }}:{{ props.item.serverPort }}
        </p>
      </div>

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
        class="flex-shrink-0 text-muted-foreground transition-transform duration-200"
        :class="{ 'rotate-180': isExpanded }"
      >
        <path d="m6 9 6 6 6-6" />
      </svg>
    </button>

    <div
      v-show="isExpanded"
      class="border-t border-border px-4 pb-4 pt-4"
    >
      <div class="space-y-3">
        <div class="flex items-center gap-2">
          <label class="text-xs text-muted-foreground whitespace-nowrap">{{ t('server.form.title') }}</label>
          <input
            v-model="editData.title"
            type="text"
            class="w-1/2 rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            :placeholder="t('server.form.titlePlaceholder')"
          />
          <button
            class="inline-flex items-center justify-center gap-1 rounded-lg bg-blue-500 px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
            @click="handleSave"
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
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
              <polyline points="17 21 17 13 7 13 7 21" />
              <polyline points="7 3 7 8 15 8" />
            </svg>
            {{ t('common.save') }}
          </button>
          <button
            class="inline-flex items-center justify-center rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm font-medium text-destructive transition-colors hover:bg-destructive/20 focus:outline-none focus:ring-2 focus:ring-destructive focus:ring-offset-2"
            @click="handleDelete"
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
            </svg>
          </button>
        </div>

        <div class="flex items-center gap-2">
          <label class="text-xs text-muted-foreground whitespace-nowrap">{{ t('server.form.address') }}</label>
          <input
            v-model="editData.serverAddr"
            type="text"
            class="flex-1 rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            :placeholder="t('server.form.addressPlaceholder')"
          />
          <label class="text-xs text-muted-foreground whitespace-nowrap">{{ t('server.form.port') }}</label>
          <input
            v-model.number="editData.serverPort"
            type="number"
            class="w-24 rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            :placeholder="t('server.form.portPlaceholder')"
          />
        </div>

        <div class="flex items-center gap-2">
          <label class="text-xs text-muted-foreground whitespace-nowrap">{{ t('server.form.authMethod') }}</label>
          <input
            v-model="editData.authMethod"
            type="text"
            class="w-32 rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            :placeholder="t('server.form.authMethodPlaceholder')"
          />
          <label class="text-xs text-muted-foreground whitespace-nowrap">{{ t('server.form.authToken') }}</label>
          <div class="relative flex-1">
            <input
              v-model="editData.authToken"
              :type="showAuthToken ? 'text' : 'password'"
              class="w-full rounded-lg border border-border bg-background px-3 py-2 pr-10 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              :placeholder="t('server.form.authTokenPlaceholder')"
            />
            <button
              type="button"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
              @click="showAuthToken = !showAuthToken"
            >
              <svg
                v-if="!showAuthToken"
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
                <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
                <circle cx="12" cy="12" r="3" />
              </svg>
              <svg
                v-else
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
                <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24" />
                <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68" />
                <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61" />
                <line x1="2" x2="22" y1="2" y2="22" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
