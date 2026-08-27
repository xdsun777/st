<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getBankStats, getStats } from "../api";
import type { BankStats, StatsSummary } from "../types";

const stats = ref<StatsSummary | null>(null);
const bankStats = ref<BankStats[]>([]);
const errorMsg = ref("");

async function refresh() {
  errorMsg.value = "";
  try {
    stats.value = await getStats();
    bankStats.value = await getBankStats();
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

    <div v-if="stats" class="space-y-4">
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

      <!-- 分题库统计 -->
      <div v-if="bankStats.length > 0">
        <h3 class="mb-2 text-sm font-medium text-gray-600">分题库统计</h3>
        <div class="overflow-x-auto rounded-xl border border-gray-200 bg-white">
          <table class="w-full min-w-[36rem] text-sm">
            <thead>
              <tr class="border-b border-gray-100 text-left text-xs text-gray-400">
                <th class="px-4 py-2.5 font-normal">题库集</th>
                <th class="px-4 py-2.5 font-normal">总题量</th>
                <th class="px-4 py-2.5 font-normal">刷题量</th>
                <th class="px-4 py-2.5 font-normal">正确</th>
                <th class="px-4 py-2.5 font-normal">错误</th>
                <th class="px-4 py-2.5 font-normal">正确率</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="row in bankStats"
                :key="row.bank_id"
                class="border-b border-gray-50 last:border-0"
              >
                <td class="px-4 py-2.5 font-medium">{{ row.bank_name }}</td>
                <td class="px-4 py-2.5 text-gray-600">{{ row.total_questions }}</td>
                <td class="px-4 py-2.5 text-gray-600">{{ row.total_records }}</td>
                <td class="px-4 py-2.5 text-green-600">{{ row.correct_count }}</td>
                <td class="px-4 py-2.5 text-red-600">{{ row.wrong_count }}</td>
                <td class="px-4 py-2.5 text-gray-600">{{ row.correct_rate }}%</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </section>
</template>
