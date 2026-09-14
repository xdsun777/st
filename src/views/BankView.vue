<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import {
  createBank,
  deleteBank,
  deleteQuestion,
  getBanks,
  getQuestions,
  renameBank,
} from "../api";
import type { Question, QuestionBank, QuestionType } from "../types";
import ConfirmDialog from "../components/ConfirmDialog.vue";
import TagFilter from "../components/TagFilter.vue";
import TagManager from "../components/TagManager.vue";
import QuestionFormModal from "../components/QuestionFormModal.vue";
import CsvImportModal from "../components/CsvImportModal.vue";

const TYPE_LABELS: Record<QuestionType, string> = {
  single: "单选",
  multi: "多选",
  judge: "判断",
  essay: "简答",
  fill: "填空",
};

const banks = ref<QuestionBank[]>([]);
const selectedBankId = ref<number | null>(null);
const questions = ref<Question[]>([]);
const tagId = ref<number | null>(null);
const loading = ref(false);
const errorMsg = ref("");

const newBankName = ref("");
const editingBankId = ref<number | null>(null);
const editingBankName = ref("");

// 弹窗状态
const questionFormOpen = ref(false);
const editingQuestion = ref<Question | null>(null);
const csvImportOpen = ref(false);
const tagManagerOpen = ref(false);
const confirmState = ref<{ type: "bank" | "question"; id: number; name: string } | null>(null);

const selectedBank = computed(
  () => banks.value.find((b) => b.id === selectedBankId.value) ?? null,
);

async function refreshBanks() {
  errorMsg.value = "";
  try {
    banks.value = await getBanks();
    if (
      selectedBankId.value !== null &&
      !banks.value.some((b) => b.id === selectedBankId.value)
    ) {
      selectedBankId.value = null;
    }
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function refreshQuestions() {
  if (selectedBankId.value === null) {
    questions.value = [];
    return;
  }
  loading.value = true;
  errorMsg.value = "";
  try {
    questions.value = await getQuestions(selectedBankId.value, tagId.value);
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function handleCreateBank() {
  const name = newBankName.value.trim();
  if (!name) return;
  errorMsg.value = "";
  try {
    const bank = await createBank(name);
    newBankName.value = "";
    await refreshBanks();
    selectedBankId.value = bank.id;
    await refreshQuestions();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

function startRename(bank: QuestionBank) {
  editingBankId.value = bank.id;
  editingBankName.value = bank.name;
}

async function handleRenameConfirm(bankId: number) {
  const name = editingBankName.value.trim();
  if (!name) return;
  errorMsg.value = "";
  try {
    await renameBank(bankId, name);
    editingBankId.value = null;
    await refreshBanks();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

function askDeleteBank(bank: QuestionBank) {
  confirmState.value = {
    type: "bank",
    id: bank.id,
    name: bank.name,
  };
}

function askDeleteQuestion(question: Question) {
  confirmState.value = {
    type: "question",
    id: question.id,
    name: question.content.slice(0, 20),
  };
}

async function handleConfirm() {
  if (!confirmState.value) return;
  const { type, id } = confirmState.value;
  errorMsg.value = "";
  try {
    if (type === "bank") {
      await deleteBank(id);
      await refreshBanks();
      await refreshQuestions();
    } else {
      await deleteQuestion(id);
      await refreshQuestions();
      await refreshBanks();
    }
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    confirmState.value = null;
  }
}

function openCreateQuestion() {
  editingQuestion.value = null;
  questionFormOpen.value = true;
}

function openEditQuestion(question: Question) {
  editingQuestion.value = question;
  questionFormOpen.value = true;
}

function onQuestionSaved() {
  refreshQuestions();
  refreshBanks();
}

function onTagChanged() {
  refreshQuestions();
  refreshBanks();
}

watch(tagId, () => refreshQuestions());

onMounted(async () => {
  await refreshBanks();
  if (banks.value.length > 0) {
    selectedBankId.value = banks.value[0].id;
  }
  await refreshQuestions();
});
</script>

<template>
  <section>
    <div class="mb-4 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
      <div>
        <h2 class="text-xl font-semibold">题库管理</h2>
        <p class="mt-1 text-sm text-gray-500">
          题库集、题目、标签与题库文件导入。删除题库集将一并删除其中所有题目。
        </p>
      </div>
      <div class="flex flex-wrap gap-2">
        <input
          v-model="newBankName"
          class="w-full min-w-0 flex-1 rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm outline-none focus:border-blue-400 sm:w-44 sm:flex-none"
          placeholder="新题库集名称"
          @keyup.enter="handleCreateBank"
        />
        <button
          class="shrink-0 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-blue-700"
          @click="handleCreateBank"
        >
          新建题库集
        </button>
        <button
          class="shrink-0 rounded-lg border border-gray-300 px-4 py-2 text-sm text-gray-600 transition hover:bg-gray-50"
          @click="csvImportOpen = true"
        >
          导入题库
        </button>
        <button
          class="shrink-0 rounded-lg border border-gray-300 px-4 py-2 text-sm text-gray-600 transition hover:bg-gray-50"
          @click="tagManagerOpen = true"
        >
          标签管理
        </button>
      </div>
    </div>

    <p v-if="errorMsg" class="mb-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
      {{ errorMsg }}
    </p>

    <!-- 题库集列表 -->
    <div
      v-if="banks.length === 0"
      class="rounded-xl border-2 border-dashed border-gray-200 bg-white p-10 text-center text-sm text-gray-400"
    >
      暂无题库集，先在上方新建一个，或通过题库文件导入题目。
    </div>
    <ul v-else class="mb-4 flex flex-wrap gap-2">
      <li
        v-for="bank in banks"
        :key="bank.id"
        class="rounded-xl border bg-white px-3 py-2 transition"
        :class="
          selectedBankId === bank.id
            ? 'border-blue-400 bg-blue-50/60'
            : 'border-gray-200 hover:border-gray-300'
        "
      >
        <div v-if="editingBankId !== bank.id" class="flex items-center gap-2">
          <button class="text-left" @click="selectedBankId = bank.id">
            <span class="text-sm font-medium">{{ bank.name }}</span>
            <span class="ml-1.5 text-xs text-gray-400">{{ bank.question_count }} 题</span>
          </button>
          <button class="text-xs text-gray-400 hover:text-blue-600" @click="startRename(bank)">
            重命名
          </button>
          <button class="text-xs text-red-400 hover:text-red-600" @click="askDeleteBank(bank)">
            删除
          </button>
        </div>
        <div v-else class="flex items-center gap-1">
          <input
            v-model="editingBankName"
            class="w-32 rounded border border-gray-300 px-2 py-1 text-sm outline-none focus:border-blue-400"
            @keyup.enter="handleRenameConfirm(bank.id)"
          />
          <button
            class="rounded bg-blue-500 px-2 py-1 text-xs text-white"
            @click="handleRenameConfirm(bank.id)"
          >
            保存
          </button>
          <button class="px-1 text-xs text-gray-500" @click="editingBankId = null">取消</button>
        </div>
      </li>
    </ul>

    <!-- 题目区 -->
    <template v-if="selectedBank">
      <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
        <TagFilter v-model="tagId" />
        <div class="flex gap-2">
          <button
            class="rounded-lg bg-blue-600 px-3 py-2 text-sm font-medium text-white transition hover:bg-blue-700"
            @click="openCreateQuestion"
          >
            新增题目
          </button>
        </div>
      </div>

      <p v-if="loading" class="text-sm text-gray-400">加载中…</p>
      <div
        v-else-if="questions.length === 0"
        class="rounded-xl border-2 border-dashed border-gray-200 bg-white p-10 text-center text-sm text-gray-400"
      >
        「{{ selectedBank.name }}」暂无题目，点击右上角新增题目或导入题库。
      </div>
      <ul v-else class="space-y-2">
        <li
          v-for="q in questions"
          :key="q.id"
          class="rounded-xl border border-gray-200 bg-white p-4"
        >
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0 flex-1">
              <div class="flex flex-wrap items-center gap-2">
                <span class="rounded bg-gray-100 px-2 py-0.5 text-xs text-gray-500">
                  {{ TYPE_LABELS[q.q_type] }}
                </span>
                <span
                  v-for="tag in q.tags"
                  :key="tag"
                  class="rounded-full bg-blue-50 px-2 py-0.5 text-xs text-blue-600"
                >
                  {{ tag }}
                </span>
              </div>
              <div class="mt-2 text-sm font-medium">{{ q.content }}</div>
              <div class="mt-1 text-xs text-gray-500">
                答案：{{ q.answer }}<span v-if="q.analysis">　解析：{{ q.analysis }}</span>
              </div>
            </div>
            <div class="flex shrink-0 gap-1.5">
              <button
                class="rounded px-2 py-1 text-xs text-gray-500 hover:text-blue-600"
                @click="openEditQuestion(q)"
              >
                编辑
              </button>
              <button
                class="rounded px-2 py-1 text-xs text-red-500 hover:text-red-600"
                @click="askDeleteQuestion(q)"
              >
                删除
              </button>
            </div>
          </div>
        </li>
      </ul>
    </template>

    <!-- 弹窗 -->
    <QuestionFormModal
      :open="questionFormOpen"
      :bank-id="selectedBankId ?? 0"
      :question="editingQuestion"
      @close="questionFormOpen = false"
      @saved="onQuestionSaved"
    />
    <CsvImportModal
      :open="csvImportOpen"
      :bank-id="selectedBankId ?? 0"
      :bank-name="selectedBank?.name ?? ''"
      @close="csvImportOpen = false"
      @imported="onQuestionSaved"
    />
    <TagManager
      :open="tagManagerOpen"
      @close="tagManagerOpen = false"
      @changed="onTagChanged"
    />
    <ConfirmDialog
      :open="confirmState !== null"
      :title="confirmState?.type === 'bank' ? '删除题库集' : '删除题目'"
      :message="
        confirmState?.type === 'bank'
          ? `将删除题库集「${confirmState?.name}」及其中的全部题目，此操作不可恢复。确认删除？`
          : `将删除题目「${confirmState?.name}」及其做题记录，此操作不可恢复。确认删除？`
      "
      confirm-text="确认删除"
      danger
      @confirm="handleConfirm"
      @cancel="confirmState = null"
    />
  </section>
</template>
