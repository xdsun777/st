<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { readFile, readTextFile } from "@tauri-apps/plugin-fs";
import { batchInsertQuestions } from "../api";
import { parseCsv, parseExcel } from "../utils/csv";
import type { CsvParseError } from "../utils/csv";
import type { NewQuestion } from "../types";

/**
 * 题库导入弹窗（技术文档 5.1）：
 * 选择 .csv / .xlsx / .xls 文件 → 按类型读取解析校验 → 错误行展示 → 确认导入 Rust 批量写入。
 * Excel 字段规范与 CSV 完全一致。
 */
const props = defineProps<{
  open: boolean;
  bankId: number;
  bankName: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "imported"): void;
}>();

const fileName = ref("");
const validCount = ref(0);
const errors = ref<CsvParseError[]>([]);
const parsedRows = ref<NewQuestion[]>([]);
const importing = ref(false);
const importResult = ref<string>("");
const errorMsg = ref("");

const canImport = computed(() => parsedRows.value.length > 0 && !importing.value);

watch(
  () => props.open,
  (open) => {
    if (open) reset();
  },
);

async function chooseFile() {
  errorMsg.value = "";
  importResult.value = "";
  const selected = await openDialog({
    multiple: false,
    filters: [{ name: "题库文件（CSV / Excel）", extensions: ["csv", "xlsx", "xls"] }],
  });
  if (!selected || typeof selected !== "string") return;

  fileName.value = selected;
  try {
    const ext = selected.split(".").pop()?.toLowerCase();
    let result;
    if (ext === "csv") {
      result = parseCsv(await readTextFile(selected));
    } else if (ext === "xlsx" || ext === "xls") {
      result = parseExcel(await readFile(selected));
    } else {
      throw new Error(`不支持的文件类型：.${ext}（仅支持 .csv / .xlsx / .xls）`);
    }
    parsedRows.value = result.rows;
    errors.value = result.errors;
    validCount.value = result.rows.length;
  } catch (e) {
    errorMsg.value = `文件读取/解析失败：${String(e)}`;
  }
}

async function handleImport() {
  if (parsedRows.value.length === 0) return;
  importing.value = true;
  errorMsg.value = "";
  importResult.value = "";
  try {
    const result = await batchInsertQuestions(props.bankId, parsedRows.value);
    const detail =
      result.skipped_details.length > 0
        ? `\n\n跳过明细：\n${result.skipped_details.join("\n")}`
        : "";
    importResult.value = `成功导入 ${result.inserted} 道，跳过 ${result.skipped} 道。${detail}`;
    emit("imported");
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    importing.value = false;
  }
}

function reset() {
  fileName.value = "";
  validCount.value = 0;
  errors.value = [];
  parsedRows.value = [];
  importResult.value = "";
  errorMsg.value = "";
}
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
    @click.self="emit('close')"
  >
    <div class="flex max-h-[85vh] w-full max-w-lg flex-col rounded-xl bg-white shadow-xl">
      <div class="flex items-center justify-between border-b border-gray-100 px-5 py-4">
        <h3 class="text-base font-semibold">题库导入</h3>
        <button class="text-gray-400 hover:text-gray-600" @click="emit('close')">关闭</button>
      </div>

      <div class="flex-1 space-y-3 overflow-y-auto px-5 py-4">
        <div class="rounded-lg bg-gray-50 px-3 py-2 text-xs text-gray-500">
          <template v-if="bankName">
            默认题库集：<span class="font-medium text-gray-700">{{ bankName }}</span>
            <br />文件含「所属题库集」列时，将按该列自动归入/创建题库集。
          </template>
          <template v-else>
            未选择题库集：将使用文件的「所属题库集」列自动创建/归入题库集；
            <br />无该列的行将跳过。
          </template>
          <br />字段规范：题目类型(必填)、题干(必填)、正确答案(必填)、选项/解析/标签/所属题库集(可选)。
          错误行将跳过并展示。
        </div>

        <div class="flex items-center gap-3">
          <button
            class="rounded-lg border border-gray-300 px-4 py-2 text-sm text-gray-600 hover:bg-gray-50"
            @click="chooseFile"
          >
            选择文件
          </button>
          <span class="min-w-0 truncate text-sm text-gray-500">
            {{ fileName || "未选择文件（支持 .csv / .xlsx / .xls）" }}
          </span>
        </div>

        <div v-if="fileName" class="rounded-lg bg-blue-50 px-3 py-2 text-sm text-blue-700">
          解析完成：合法 {{ validCount }} 行，错误 {{ errors.length }} 行。
        </div>

        <div v-if="errors.length > 0" class="rounded-lg border border-red-100 bg-red-50 p-3">
          <div class="mb-1 text-xs font-medium text-red-600">错误行（不导入）：</div>
          <ul class="max-h-32 space-y-0.5 overflow-y-auto text-xs text-red-500">
            <li v-for="(err, index) in errors" :key="index">第 {{ err.row }} 行：{{ err.message }}</li>
          </ul>
        </div>

        <p v-if="importResult" class="whitespace-pre-line rounded-lg bg-green-50 px-3 py-2 text-xs text-green-700">
          {{ importResult }}
        </p>
        <p v-if="errorMsg" class="rounded-lg bg-red-50 px-3 py-2 text-xs text-red-600">
          {{ errorMsg }}
        </p>
      </div>

      <div class="flex justify-end gap-2 border-t border-gray-100 px-5 py-3">
        <button
          class="rounded-lg border border-gray-300 px-4 py-2 text-sm text-gray-600 hover:bg-gray-50"
          @click="
            reset();
            emit('close');
          "
        >
          取消
        </button>
        <button
          class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
          :disabled="!canImport"
          @click="handleImport"
        >
          {{ importing ? "导入中…" : `确认导入 ${validCount} 道` }}
        </button>
      </div>
    </div>
  </div>
</template>
