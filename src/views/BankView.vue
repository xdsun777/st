<script setup lang="ts">
import { onMounted, ref } from "vue";
import { createBank, deleteBank, getBanks } from "../api";
import type { QuestionBank } from "../types";

const banks = ref<QuestionBank[]>([]);
const loading = ref(false);
const errorMsg = ref("");
const newBankName = ref("");
const confirmDeleteId = ref<number | null>(null);

async function refresh() {
  loading.value = true;
  errorMsg.value = "";
  try {
    banks.value = await getBanks();
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function handleCreate() {
  const name = newBankName.value.trim();
  if (!name) return;
  errorMsg.value = "";
  try {
    await createBank(name);
    newBankName.value = "";
    await refresh();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

/** 高危操作：删除题库集需二次确认（业务文档 7.5） */
function askDelete(id: number) {
  confirmDeleteId.value = id;
}

async function handleDeleteConfirm(id: number) {
  errorMsg.value = "";
  try {
    await deleteBank(id);
    confirmDeleteId.value = null;
    await refresh();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

onMounted(refresh);
</script>

<template>
  <section>
    <div class="mb-4 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
      <div>
        <h2 class="text-xl font-semibold">题库管理</h2>
        <p class="mt-1 text-sm text-gray-500">
          题库集、题目、标签、CSV 导入。初始化阶段已打通题库集增删查，其余功能开发中。
        </p>
      </div>
      <div class="flex gap-2">
        <input
          v-model="newBankName"
          class="w-full min-w-0 flex-1 rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm outline-none focus:border-blue-400 sm:w-44 sm:flex-none"
          placeholder="新题库集名称"
          @keyup.enter="handleCreate"
        />
        <button
          class="shrink-0 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-blue-700"
          @click="handleCreate"
        >
          新建题库集
        </button>
      </div>
    </div>

    <p v-if="errorMsg" class="mb-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
      {{ errorMsg }}
    </p>
    <p v-if="loading" class="text-sm text-gray-400">加载中…</p>

    <div
      v-else-if="banks.length === 0"
      class="rounded-xl border-2 border-dashed border-gray-200 bg-white p-10 text-center text-sm text-gray-400"
    >
      暂无题库集，先在上方新建一个；CSV 导入题库功能开发中。
    </div>

    <ul v-else class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <li
        v-for="bank in banks"
        :key="bank.id"
        class="rounded-xl border border-gray-200 bg-white p-4"
      >
        <div class="flex items-center justify-between gap-2">
          <div class="min-w-0">
            <div class="truncate font-medium">{{ bank.name }}</div>
            <div class="mt-1 text-xs text-gray-400">{{ bank.question_count }} 道题</div>
          </div>
          <div v-if="confirmDeleteId !== bank.id" class="shrink-0">
            <button
              class="rounded px-2 py-1.5 text-xs text-red-500 hover:text-red-600"
              @click="askDelete(bank.id)"
            >
              删除
            </button>
          </div>
          <div v-else class="flex shrink-0 flex-wrap items-center justify-end gap-2">
            <span class="text-xs text-gray-500">删除后题目一并删除，确认？</span>
            <button
              class="rounded bg-red-500 px-2 py-1.5 text-xs text-white hover:bg-red-600"
              @click="handleDeleteConfirm(bank.id)"
            >
              确认删除
            </button>
            <button class="rounded px-2 py-1.5 text-xs text-gray-500" @click="confirmDeleteId = null">
              取消
            </button>
          </div>
        </div>
      </li>
    </ul>
  </section>
</template>
