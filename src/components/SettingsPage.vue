<script setup lang="ts">
import { ref } from "vue";

type SettingsTab = "general" | "advanced" | "about";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const activeTab = ref<SettingsTab>("general");
const language = ref("zh-CN");
const theme = ref("system");

const languages = [
  { value: "zh-CN", label: "简体中文" },
  { value: "zh-TW", label: "繁體中文" },
  { value: "en", label: "English" },
  { value: "ja", label: "日本語" },
];

const themes = [
  { value: "light", label: "浅色", icon: "☀️" },
  { value: "dark", label: "深色", icon: "🌙" },
  { value: "system", label: "跟随系统", icon: "💻" },
];
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="flex items-center gap-3 px-4 py-3 border-b border-border">
      <button
        class="inline-flex items-center justify-center h-8 w-8 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        @click="emit('close')"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="m12 19-7-7 7-7" />
          <path d="M19 12H5" />
        </svg>
      </button>
      <h1 class="text-lg font-semibold">设置</h1>
    </header>

    <div class="flex gap-1 p-2 border-b border-border">
      <button
        class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
        :class="activeTab === 'general' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted hover:text-foreground'"
        @click="activeTab = 'general'"
      >
        通用
      </button>
      <button
        class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
        :class="activeTab === 'advanced' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted hover:text-foreground'"
        @click="activeTab = 'advanced'"
      >
        高级
      </button>
      <button
        class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
        :class="activeTab === 'about' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted hover:text-foreground'"
        @click="activeTab = 'about'"
      >
        关于
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-4">
      <div v-if="activeTab === 'general'" class="space-y-6">
        <div>
          <h3 class="text-sm font-medium mb-2">界面语言</h3>
          <p class="text-xs text-muted-foreground mb-3">切换后立即预览界面语言，保存后永久生效。</p>
          <div class="flex gap-2">
            <button
              v-for="lang in languages"
              :key="lang.value"
              class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
              :class="language === lang.value ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground hover:bg-muted/80'"
              @click="language = lang.value"
            >
              {{ lang.label }}
            </button>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium mb-2">外观主题</h3>
          <p class="text-xs text-muted-foreground mb-3">选择应用的外观主题，立即生效。</p>
          <div class="flex gap-2">
            <button
              v-for="t in themes"
              :key="t.value"
              class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
              :class="theme === t.value ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground hover:bg-muted/80'"
              @click="theme = t.value"
            >
              {{ t.label }}
            </button>
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'advanced'" class="space-y-6">
        <div>
          <h3 class="text-sm font-medium mb-2">高级设置</h3>
          <p class="text-xs text-muted-foreground">高级设置内容待实现。</p>
        </div>
      </div>

      <div v-else-if="activeTab === 'about'" class="space-y-6">
        <div>
          <h3 class="text-sm font-medium mb-2">关于</h3>
          <p class="text-xs text-muted-foreground">frpc tray v0.1.0</p>
        </div>
      </div>
    </div>
  </div>
</template>
