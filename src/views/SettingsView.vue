<script setup lang="ts">
import { onMounted, ref } from "vue";
import { clearApiKey, getAiConfig, getSetting, saveAiConfig, setSetting } from "../api";
import { resetAiClient, testAiConnectionJs } from "../utils/ai";
import { useAppStore, type ThemeMode } from "../stores/app";

/**
 * 设置页：主题三态 + AI 配置（OpenAI 兼容）+ 激励开关。
 * API Key 密文显示；留空表示不修改。
 */
const store = useAppStore();

const themes: { value: ThemeMode; label: string; desc: string }[] = [
  { value: "system", label: "跟随系统", desc: "随系统深色模式" },
  { value: "light", label: "浅色", desc: "始终浅色" },
  { value: "dark", label: "深色", desc: "夜间刷题友好" },
];

// 主题
const themeMode = ref<ThemeMode>(store.themeMode);
// AI 配置
const baseUrl = ref("https://api.deepseek.com");
const apiKey = ref("");
const apiKeyMasked = ref("");
const model = ref("deepseek-v4-pro");
const judgeEnabled = ref(false);
const analysisEnabled = ref(false);
// 激励开关
const encourageEnabled = ref(true);

const busy = ref(false);
const message = ref("");
const errorMsg = ref("");

async function chooseTheme(mode: ThemeMode) {
  themeMode.value = mode;
  store.applyTheme(mode, matchMedia("(prefers-color-scheme: dark)").matches);
  try {
    await setSetting("theme_mode", mode);
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function loadConfig() {
  try {
    const cfg = await getAiConfig();
    baseUrl.value = cfg.base_url || "https://api.deepseek.com";
    model.value = cfg.model || "deepseek-v4-pro";
    apiKeyMasked.value = cfg.api_key_masked;
    judgeEnabled.value = cfg.judge_enabled;
    analysisEnabled.value = cfg.analysis_enabled;
  } catch (e) {
    errorMsg.value = String(e);
  }
  try {
    encourageEnabled.value = (await getSetting("encourage_enabled")) !== "0";
  } catch {
    // 默认开启
  }
}

async function handleSaveAi() {
  busy.value = true;
  errorMsg.value = "";
  message.value = "";
  try {
    await saveAiConfig({
      base_url: baseUrl.value.trim(),
      api_key: apiKey.value.trim(),
      model: model.value.trim(),
      judge_enabled: judgeEnabled.value,
      analysis_enabled: analysisEnabled.value,
    });
    resetAiClient();
    apiKey.value = "";
    message.value = "AI 配置已保存";
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function handleToggleEncourage() {
  try {
    await setSetting("encourage_enabled", encourageEnabled.value ? "1" : "0");
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function handleTestConnection() {
  busy.value = true;
  errorMsg.value = "";
  message.value = "";
  try {
    // 测试连接前先保存未保存的 AI 配置
    await saveAiConfig({
      base_url: baseUrl.value.trim(),
      api_key: apiKey.value.trim(),
      model: model.value.trim(),
      judge_enabled: judgeEnabled.value,
      analysis_enabled: analysisEnabled.value,
    });
    resetAiClient();
    apiKey.value = "";
    const result = await testAiConnectionJs();
    message.value = result;
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function handleClearKey() {
  errorMsg.value = "";
  message.value = "";
  try {
    await clearApiKey();
    apiKey.value = "";
    apiKeyMasked.value = "";
    resetAiClient();
    message.value = "API Key 已清空，AI 功能已恢复纯离线";
  } catch (e) {
    errorMsg.value = String(e);
  }
}

onMounted(() => {
  themeMode.value = store.themeMode;
  loadConfig();
});
</script>

<template>
  <section>
    <h2 class="text-xl font-semibold">设置</h2>
    <p class="mb-4 mt-1 text-sm text-gray-500">
      主题与 AI 配置（AI 默认关闭，配置后仅在启用场景发起网络请求）。
    </p>

    <!-- 主题 -->
    <div class="mb-4 rounded-xl border border-gray-200 bg-white p-4">
      <h3 class="mb-2 text-sm font-medium">主题</h3>
      <div class="grid grid-cols-3 gap-2">
        <button
          v-for="t in themes"
          :key="t.value"
          class="rounded-lg border px-3 py-2.5 text-left transition"
          :class="
            themeMode === t.value
              ? 'border-blue-400 bg-blue-50 text-blue-700'
              : 'border-gray-200 text-gray-600 hover:border-gray-300'
          "
          @click="chooseTheme(t.value)"
        >
          <div class="text-sm font-medium">{{ t.label }}</div>
          <div class="mt-0.5 text-xs text-gray-400">{{ t.desc }}</div>
        </button>
      </div>
    </div>

    <!-- AI 配置 -->
    <div class="mb-4 rounded-xl border border-gray-200 bg-white p-4">
      <h3 class="mb-3 text-sm font-medium">AI 配置（OpenAI 兼容协议）</h3>
      <div class="space-y-3">
        <div>
          <label class="mb-1 block text-xs text-gray-500">Base URL（默认 https://api.deepseek.com）</label>
          <input
            v-model="baseUrl"
            class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm outline-none focus:border-blue-400"
            placeholder="https://api.deepseek.com"
          />
        </div>
        <div>
          <label class="mb-1 block text-xs text-gray-500">API Key（密文显示，留空不修改）</label>
          <input
            v-model="apiKey"
            type="password"
            class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm outline-none focus:border-blue-400"
            :placeholder="apiKeyMasked ? `已保存：${apiKeyMasked}` : 'sk-…'"
          />
          <p class="mt-1 text-xs text-gray-400">
            密钥明文存储于本机数据库，请勿在他人设备上配置。
          </p>
          <button
            v-if="apiKeyMasked"
            class="mt-1 rounded-lg border border-gray-300 px-2 py-1 text-xs text-gray-500 transition hover:border-red-300 hover:text-red-600"
            @click="handleClearKey"
          >
            清空 Key
          </button>
        </div>
        <div>
          <label class="mb-1 block text-xs text-gray-500">模型名</label>
          <input
            v-model="model"
            class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm outline-none focus:border-blue-400"
            placeholder="deepseek-v4-pro"
          />
        </div>
        <div class="flex flex-wrap gap-4">
          <label class="flex items-center gap-2 text-sm text-gray-600">
            <input v-model="judgeEnabled" type="checkbox" class="h-4 w-4" />
            启用 AI 判题（简答）
          </label>
          <label class="flex items-center gap-2 text-sm text-gray-600">
            <input v-model="analysisEnabled" type="checkbox" class="h-4 w-4" />
            启用 AI 错题解析
          </label>
        </div>
        <div class="flex gap-2">
          <button
            class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-blue-700 disabled:opacity-50"
            :disabled="busy"
            @click="handleSaveAi"
          >
            保存 AI 配置
          </button>
          <button
            class="rounded-lg border border-gray-300 px-4 py-2 text-sm text-gray-600 transition hover:bg-gray-50 disabled:opacity-50"
            :disabled="busy"
            @click="handleTestConnection"
          >
            {{ busy ? "处理中…" : "测试连接" }}
          </button>
        </div>
      </div>
    </div>

    <!-- 激励 -->
    <div class="mb-4 rounded-xl border border-gray-200 bg-white p-4">
      <h3 class="mb-2 text-sm font-medium">刷题体验</h3>
      <label class="flex items-center gap-2 text-sm text-gray-600">
        <input
          v-model="encourageEnabled"
          type="checkbox"
          class="h-4 w-4"
          @change="handleToggleEncourage"
        />
        连续答对时显示轻量激励文案
      </label>
    </div>

    <p v-if="message" class="mb-3 rounded-lg bg-green-50 px-3 py-2 text-sm text-green-700">
      {{ message }}
    </p>
    <p v-if="errorMsg" class="mb-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
      {{ errorMsg }}
    </p>
  </section>
</template>
