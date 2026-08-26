<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getStats } from "../api";
import type { StatsSummary } from "../types";

const stats = ref<StatsSummary | null>(null);
const errorMsg = ref("");

async function refresh() {
  errorMsg.value = "";
  try {
    stats.value = await getStats();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

onMounted(refresh);
</script>

<template>
  <section>
    <h2 class="text-xl font-semibold">统计</h2>
    <p class="mb-4 mt-1 text-sm text-gray-500">
      统计全部由 Rust 层 SQL 聚合计算（技术文档 5.4）。
    </p>

    <p v-if="errorMsg" class="mb-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
      {{ errorMsg }}
    </p>

    <div v-if="stats" class="space-y-3">
      <div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <div class="rounded-xl border border-gray-200 bg-white p-4">
          <div class="text-xs text-gray-400">总题量</div>
          <div class="mt-1 text-2xl font-semibold">{{ stats.total_questions }}</div>
        </div>
        <div class="rounded-xl border border-gray-200 bg-white p-4">
          <div class="text-xs text-gray-400">总刷题量</div>
          <div class="mt-1 text-2xl font-semibold">{{ stats.total_records }}</div>
        </div>
        <div class="rounded-xl border border-gray-200 bg-white p-4">
          <div class="text-xs text-gray-400">正确数</div>
          <div class="mt-1 text-2xl font-semibold text-green-600">
            {{ stats.correct_count }}
          </div>
        </div>
        <div class="rounded-xl border border-gray-200 bg-white p-4">
          <div class="text-xs text-gray-400">错误数</div>
          <div class="mt-1 text-2xl font-semibold text-red-600">{{ stats.wrong_count }}</div>
        </div>
      </div>
      <div class="rounded-xl border border-gray-200 bg-white p-4">
        <div class="text-xs text-gray-400">整体正确率</div>
        <div class="mt-1 text-2xl font-semibold">{{ stats.correct_rate }}%</div>
      </div>
    </div>
  </section>
</template>
