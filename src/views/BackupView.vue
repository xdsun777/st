<script setup lang="ts">
import { ref } from "vue";
import { open as openDialog, save } from "@tauri-apps/plugin-dialog";
import { exportBackup, importBackup } from "../api";
import type { BackupResult } from "../types";
import ConfirmDialog from "../components/ConfirmDialog.vue";

const busy = ref(false);
const message = ref("");
const errorMsg = ref("");
const pendingImportPath = ref("");
const confirmOpen = ref(false);

async function handleExport() {
  errorMsg.value = "";
  message.value = "";
  const date = new Date();
  const stamp = `${date.getFullYear()}${String(date.getMonth() + 1).padStart(2, "0")}${String(
    date.getDate(),
  ).padStart(2, "0")}-${String(date.getHours()).padStart(2, "0")}${String(
    date.getMinutes(),
  ).padStart(2, "0")}`;
  const selected = await save({
    defaultPath: `st-backup-${stamp}.qpbackup`,
    filters: [{ name: "刷题备份包", extensions: ["qpbackup"] }],
  });
  if (!selected) return;

  busy.value = true;
  try {
    const result = await exportBackup(selected);
    message.value = formatResult("导出成功", result);
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function handlePickRestore() {
  errorMsg.value = "";
  message.value = "";
  const selected = await openDialog({
    multiple: false,
    filters: [{ name: "刷题备份包", extensions: ["qpbackup"] }],
  });
  if (!selected || typeof selected !== "string") return;
  // 恢复前风险确认（业务文档 4.7：恢复前弹窗风险提示）
  pendingImportPath.value = selected;
  confirmOpen.value = true;
}

async function handleRestoreConfirm() {
  confirmOpen.value = false;
  busy.value = true;
  errorMsg.value = "";
  message.value = "";
  try {
    const result = await importBackup(pendingImportPath.value);
    message.value = formatResult("恢复成功，已覆盖本地全部数据", result);
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    busy.value = false;
    pendingImportPath.value = "";
  }
}

function formatResult(prefix: string, r: BackupResult): string {
  return `${prefix}：题库集 ${r.question_banks}、题目 ${r.questions}、标签 ${r.tags}、标签关联 ${r.question_tags}、做题记录 ${r.answer_records}、刷题会话 ${r.practice_sessions}。\n文件：${r.path}`;
}
</script>

<template>
  <section>
    <h2 class="text-xl font-semibold">备份恢复</h2>
    <p class="mb-4 mt-1 text-sm text-gray-500">
      全量备份导出 .qpbackup 数据包；恢复前必须二次确认，覆盖本地全部数据。
    </p>

    <div class="space-y-3 rounded-xl border border-gray-200 bg-white p-5">
      <div>
        <div class="text-sm font-medium">备份导出</div>
        <p class="mt-1 text-xs text-gray-400">
          导出题目、做题记录、错题标记、收藏、标签为 .qpbackup 文件。
        </p>
        <button
          class="mt-3 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-blue-700 disabled:opacity-50"
          :disabled="busy"
          @click="handleExport"
        >
          选择位置导出
        </button>
      </div>

      <div class="border-t border-gray-100 pt-3">
        <div class="text-sm font-medium">备份恢复</div>
        <p class="mt-1 text-xs text-gray-400">
          从 .qpbackup 文件恢复，将覆盖本地全部现有数据。
        </p>
        <button
          class="mt-3 rounded-lg border border-red-300 px-4 py-2 text-sm font-medium text-red-600 transition hover:bg-red-50 disabled:opacity-50"
          :disabled="busy"
          @click="handlePickRestore"
        >
          选择备份文件恢复
        </button>
      </div>

      <p v-if="busy" class="text-sm text-gray-400">处理中…</p>
      <p v-if="message" class="whitespace-pre-line rounded-lg bg-green-50 px-3 py-2 text-xs text-green-700">
        {{ message }}
      </p>
      <p v-if="errorMsg" class="rounded-lg bg-red-50 px-3 py-2 text-xs text-red-600">
        {{ errorMsg }}
      </p>
    </div>

    <ConfirmDialog
      :open="confirmOpen"
      title="恢复备份"
      :message="`将覆盖本地全部现有数据（题库、题目、做题记录、错题、收藏、标签），此操作不可恢复。\n\n备份文件：${pendingImportPath}\n\n确认继续恢复？`"
      confirm-text="覆盖并恢复"
      danger
      @confirm="handleRestoreConfirm"
      @cancel="confirmOpen = false"
    />
  </section>
</template>
