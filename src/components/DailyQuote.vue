<script setup lang="ts">
import { onMounted, ref } from "vue";

/**
 * 每日一语：调用一言接口随机返回语录，双击切换下一句。
 * 失败时静默降级，保留默认文案「每日一语」；不展示来源 from。
 */
const quote = ref("每日一语");

async function loadQuote() {
  try {
    const resp = await fetch("https://base.itab.link/yiyan/random?lang=cn");
    if (!resp.ok) return;
    const json = (await resp.json()) as { data?: { hitokoto?: unknown }; hitokoto?: unknown };
    const hitokoto = json?.data?.hitokoto ?? json?.hitokoto;
    if (typeof hitokoto === "string" && hitokoto.trim()) {
      quote.value = hitokoto;
    }
  } catch {
    // 接口失败静默降级，保留默认「每日一语」
  }
}

onMounted(loadQuote);
</script>

<template>
  <p
    class="line-clamp-2 cursor-pointer select-none text-xs text-gray-400"
    title="双击换一句"
    @dblclick="loadQuote"
  >
    {{ quote }}
  </p>
</template>
