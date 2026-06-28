<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";

export interface ProxyFormData {
  name: string;
  desc: string;
  type: string;
  localIP: string;
  localPort: number;
  remotePort: number | null;
  customDomains: string;
  locations: string;
}

const props = defineProps<{
  open: boolean;
  mode: "create" | "edit";
  initialData?: ProxyFormData;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "submit", data: ProxyFormData): void;
}>();

const { t } = useI18n();

const defaultData: ProxyFormData = {
  name: "",
  desc: "",
  type: "tcp",
  localIP: "127.0.0.1",
  localPort: 8080,
  remotePort: null,
  customDomains: "",
  locations: "/",
};

const formData = ref<ProxyFormData>({ ...defaultData });

watch(
  () => props.open,
  (val) => {
    if (val) {
      if (props.mode === "edit" && props.initialData) {
        formData.value = { ...props.initialData };
      } else {
        formData.value = { ...defaultData };
      }
    }
  }
);

function handleClose() {
  emit("update:open", false);
}

function handleSubmit() {
  if (!formData.value.name.trim()) return;
  emit("submit", { ...formData.value });
  emit("update:open", false);
}

const proxyTypes = ["tcp", "udp", "http", "https", "stcp", "sudp", "xtcp"];
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center"
    >
      <div
        class="fixed inset-0 bg-black/60 backdrop-blur-sm"
        @click="handleClose"
      />

      <div
        class="relative z-10 w-full max-w-md mx-4 rounded-2xl border border-border bg-background p-6 shadow-2xl"
      >
        <div class="flex items-center justify-between mb-6">
          <h2 class="text-lg font-semibold">
            {{ mode === "create" ? t("proxy.dialog.createTitle") : t("proxy.dialog.editTitle") }}
          </h2>
          <button
            class="inline-flex items-center justify-center h-8 w-8 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
            @click="handleClose"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M18 6 6 18" />
              <path d="m6 6 12 12" />
            </svg>
          </button>
        </div>

        <div class="space-y-4">
          <div class="space-y-1">
            <label class="text-xs text-muted-foreground">{{ t("proxy.dialog.name") }} *</label>
            <input
              v-model="formData.name"
              type="text"
              class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              :placeholder="t('proxy.dialog.namePlaceholder')"
            />
          </div>

          <div class="space-y-1">
            <label class="text-xs text-muted-foreground">{{ t("proxy.dialog.desc") }}</label>
            <input
              v-model="formData.desc"
              type="text"
              class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              :placeholder="t('proxy.dialog.descPlaceholder')"
            />
          </div>

          <div class="flex items-end gap-2">
            <div class="w-1/2 space-y-1">
              <label class="text-xs text-muted-foreground">{{ t("proxy.dialog.type") }}</label>
              <select
                v-model="formData.type"
                class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              >
                <option v-for="pt in proxyTypes" :key="pt" :value="pt">
                  {{ pt.toUpperCase() }}
                </option>
              </select>
            </div>
            <div class="w-1/2 space-y-1">
              <label class="text-xs text-muted-foreground">{{ t("proxy.dialog.localIP") }}</label>
              <input
                v-model="formData.localIP"
                type="text"
                class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
            </div>
          </div>

          <div class="flex items-end gap-2">
            <div class="w-1/2 space-y-1">
              <label class="text-xs text-muted-foreground">{{ t("proxy.dialog.localPort") }}</label>
              <input
                v-model.number="formData.localPort"
                type="number"
                class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
            </div>
            <div class="w-1/2 space-y-1">
              <label class="text-xs text-muted-foreground">{{ t("proxy.dialog.remotePort") }}</label>
              <input
                v-model.number="formData.remotePort"
                type="number"
                class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                :placeholder="formData.type === 'tcp' || formData.type === 'udp' ? t('proxy.dialog.required') : t('proxy.dialog.optional')"
              />
            </div>
          </div>

          <div class="space-y-1" v-if="formData.type === 'http' || formData.type === 'https'">
            <label class="text-xs text-muted-foreground">{{ t("proxy.dialog.customDomains") }}</label>
            <input
              v-model="formData.customDomains"
              type="text"
              class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              :placeholder="t('proxy.dialog.customDomainsPlaceholder')"
            />
          </div>

          <div class="space-y-1" v-if="formData.type === 'http'">
            <label class="text-xs text-muted-foreground">{{ t("proxy.dialog.locations") }}</label>
            <input
              v-model="formData.locations"
              type="text"
              class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              :placeholder="t('proxy.dialog.locationsPlaceholder')"
            />
          </div>
        </div>

        <div class="flex items-center justify-end gap-2 mt-6">
          <button
            class="inline-flex items-center justify-center h-9 rounded-lg border border-border px-4 text-sm font-medium text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
            @click="handleClose"
          >
            {{ t("common.cancel") }}
          </button>
          <button
            class="inline-flex items-center justify-center h-9 rounded-lg bg-blue-500 px-4 text-sm font-medium text-white transition-colors hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
            :disabled="!formData.name.trim()"
            @click="handleSubmit"
          >
            {{ mode === "create" ? t("common.add") : t("common.save") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
