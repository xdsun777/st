<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { machineJudge } from "../utils/judge";
import type { Question, QuestionType } from "../types";

/**
 * 题目作答卡片：五种题型渲染、作答、机器判分提交、填空覆写/简答手动标记。
 * 父组件通过 emitted 事件调用 Rust 命令持久化。
 */
const props = defineProps<{
  question: Question;
  index: number;
  total: number;
  /** AI 判题结果（essay；null 表示无） */
  aiJudgeResult: { correct: number; reason: string } | null;
  /** AI 判题中 */
  aiJudging: boolean;
  /** AI 错题解析文本（答错后异步生成；null 表示无） */
  aiAnalysisText: string | null;
}>();

const emit = defineEmits<{
  (e: "submitted", payload: { userAnswer: string | string[]; machineResult: number | null }): void;
  (e: "override", manualResult: number): void;
  (e: "toggle-collect"): void;
}>();

const TYPE_LABELS: Record<QuestionType, string> = {
  single: "单选题",
  multi: "多选题",
  judge: "判断题",
  essay: "简答题",
  fill: "填空题",
};

const options = computed<string[]>(() => {
  if (!props.question.options) return [];
  try {
    const parsed = JSON.parse(props.question.options);
    return Array.isArray(parsed) ? parsed.map(String) : [];
  } catch {
    return [];
  }
});

/** 刷题进度（0-100），用于顶部细进度条，替代「第 N/M 题」数字 */
const progress = computed(() => {
  if (props.total <= 0) return 0;
  return Math.min(100, Math.max(0, Math.round(((props.index + 1) / props.total) * 100)));
});

// 作答状态
const singleSelected = ref("");
const multiSelected = ref<string[]>([]);
const judgeSelected = ref("");
const textAnswer = ref("");
const submitted = ref(false);
const machineResult = ref<number | null>(null);
/** 覆写/手动标记结果：null 表示未覆写 */
const overriddenResult = ref<number | null>(null);

const isSingle = computed(() => props.question.q_type === "single");
const isMulti = computed(() => props.question.q_type === "multi");
const isJudge = computed(() => props.question.q_type === "judge");
const isFill = computed(() => props.question.q_type === "fill");
const isEssay = computed(() => props.question.q_type === "essay");

const finalResult = computed(() => {
  if (overriddenResult.value !== null) return overriddenResult.value;
  if (props.aiJudgeResult) return props.aiJudgeResult.correct;
  return machineResult.value;
});
const displayCorrect = computed(() =>
  finalResult.value === null ? null : finalResult.value === 1,
);

const canSubmit = computed(() => {
  if (submitted.value) return false;
  if (isSingle.value) return singleSelected.value !== "";
  if (isMulti.value) return multiSelected.value.length > 0;
  if (isJudge.value) return judgeSelected.value !== "";
  return textAnswer.value.trim() !== "";
});

function optionLetter(index: number): string {
  return String.fromCharCode(65 + index);
}

function toggleMulti(option: string) {
  if (submitted.value) return;
  const pos = multiSelected.value.indexOf(option);
  if (pos >= 0) {
    multiSelected.value.splice(pos, 1);
  } else {
    multiSelected.value.push(option);
  }
}

function submit() {
  if (!canSubmit.value) return;

  let userAnswer: string | string[];
  let machine: number | null;

  if (isSingle.value) {
    userAnswer = singleSelected.value;
    machine = machineJudge("single", userAnswer, props.question.answer) ? 1 : 0;
  } else if (isMulti.value) {
    userAnswer = [...multiSelected.value];
    machine = machineJudge("multi", userAnswer, props.question.answer) ? 1 : 0;
  } else if (isJudge.value) {
    userAnswer = judgeSelected.value;
    machine = machineJudge("judge", userAnswer, props.question.answer) ? 1 : 0;
  } else if (isFill.value) {
    userAnswer = textAnswer.value;
    machine = machineJudge("fill", userAnswer, props.question.answer) ? 1 : 0;
  } else {
    userAnswer = textAnswer.value;
    machine = null; // essay：无机器判分
  }

  machineResult.value = machine;
  submitted.value = true;
  emit("submitted", { userAnswer, machineResult: machine });
}

/** 填空覆写 / 简答手动标记（0错误 1正确） */
function markManual(result: number) {
  overriddenResult.value = result;
  emit("override", result);
}

function reset() {
  singleSelected.value = "";
  multiSelected.value = [];
  judgeSelected.value = "";
  textAnswer.value = "";
  submitted.value = false;
  machineResult.value = null;
  overriddenResult.value = null;
}

watch(() => props.question.id, reset);
</script>

<template>
  <div class="rounded-xl border border-gray-200 bg-white p-5 sm:p-6">
    <!-- 顶部细进度条（替代「第 N/M 题」数字，弱化完成压力） -->
    <div class="mb-4 h-1 w-full overflow-hidden rounded-full bg-gray-100">
      <div
        class="h-full rounded-full bg-blue-500 transition-all duration-300"
        :style="{ width: `${progress}%` }"
      ></div>
    </div>

    <div class="mb-4 flex items-center justify-between gap-3">
      <span class="rounded bg-blue-50 px-2 py-0.5 text-xs text-blue-600">
        {{ TYPE_LABELS[question.q_type] }}
      </span>
      <div class="flex items-center gap-3">
        <button
          class="rounded-full border px-3 py-1 text-xs transition"
          :class="
            question.is_collect === 1
              ? 'border-amber-300 bg-amber-50 text-amber-600'
              : 'border-gray-200 text-gray-400 hover:border-amber-300 hover:text-amber-500'
          "
          @click="emit('toggle-collect')"
        >
          {{ question.is_collect === 1 ? "★ 已收藏" : "☆ 收藏" }}
        </button>
      </div>
    </div>

    <h3 class="text-base font-medium leading-relaxed">{{ question.content }}</h3>

    <!-- 选项（单选/多选） -->
    <div v-if="isSingle || isMulti" class="mt-4 space-y-2">
      <button
        v-for="(option, i) in options"
        :key="i"
        class="flex w-full items-start gap-3 rounded-lg border px-3 py-2.5 text-left text-sm transition"
        :class="
          isSingle
            ? singleSelected === option
              ? 'border-blue-400 bg-blue-50 text-blue-700'
              : 'border-gray-200 hover:border-gray-300'
            : multiSelected.includes(option)
              ? 'border-blue-400 bg-blue-50 text-blue-700'
              : 'border-gray-200 hover:border-gray-300'
        "
        :disabled="submitted"
        @click="isSingle ? (singleSelected = option) : toggleMulti(option)"
      >
        <span class="font-medium">{{ optionLetter(i) }}</span>
        <span class="min-w-0 flex-1">{{ option }}</span>
        <span v-if="isMulti" class="text-xs text-gray-400">
          {{ multiSelected.includes(option) ? "已选" : "" }}
        </span>
      </button>
    </div>

    <!-- 判断 -->
    <div v-else-if="isJudge" class="mt-4 flex gap-3">
      <button
        v-for="choice in ['true', 'false']"
        :key="choice"
        class="flex-1 rounded-lg border px-4 py-3 text-sm font-medium transition"
        :class="
          judgeSelected === choice
            ? 'border-blue-400 bg-blue-50 text-blue-700'
            : 'border-gray-200 hover:border-gray-300'
        "
        :disabled="submitted"
        @click="judgeSelected = choice"
      >
        {{ choice === "true" ? "正确" : "错误" }}
      </button>
    </div>

    <!-- 填空 / 简答 -->
    <div v-else class="mt-4">
      <textarea
        v-model="textAnswer"
        rows="3"
        class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm outline-none focus:border-blue-400"
        :placeholder="isFill ? '输入答案（大小写敏感，去除首尾空格）' : '输入你的作答'"
        :disabled="submitted"
      />
    </div>

    <!-- 提交按钮 -->
    <div class="mt-4">
      <button
        v-if="!submitted"
        class="rounded-lg bg-blue-600 px-5 py-2 text-sm font-medium text-white transition hover:bg-blue-700 disabled:opacity-50"
        :disabled="!canSubmit"
        @click="submit"
      >
        提交答案
      </button>

      <!-- 提交后：结果与解析 -->
      <div v-else class="space-y-3">
        <div
          v-if="isEssay && aiJudging"
          class="rounded-lg bg-gray-50 px-3 py-2 text-sm text-gray-500"
        >
          AI 判题中…
        </div>
        <div
          v-else-if="displayCorrect !== null"
          class="rounded-lg px-3 py-2 text-sm font-medium"
          :class="displayCorrect ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-600'"
        >
          {{ displayCorrect ? "回答正确" : "回答错误" }}
          <span v-if="isEssay && aiJudgeResult" class="ml-1 text-xs font-normal opacity-70">
            （AI 判题）
          </span>
        </div>
        <div v-else class="rounded-lg bg-gray-50 px-3 py-2 text-sm text-gray-500">
          简答题无机器判分，请手动标记对错。
        </div>

        <div
          v-if="isEssay && aiJudgeResult?.reason"
          class="rounded-lg bg-blue-50/60 px-3 py-2 text-sm"
        >
          <span class="text-gray-500">AI 判题理由：</span>
          <span>{{ aiJudgeResult.reason }}</span>
        </div>

        <div class="rounded-lg bg-gray-50 px-3 py-2 text-sm">
          <span class="text-gray-500">参考答案：</span>
          <span class="font-medium">{{ question.answer }}</span>
        </div>

        <div v-if="question.analysis" class="rounded-lg bg-blue-50/60 px-3 py-2 text-sm">
          <span class="text-gray-500">解析：</span>
          <span>{{ question.analysis }}</span>
        </div>

        <div
          v-if="displayCorrect === false && aiAnalysisText"
          class="rounded-lg border border-blue-200 bg-blue-50/40 px-3 py-2 text-sm"
        >
          <div class="mb-0.5 text-xs font-medium text-blue-600">AI 错题解析（仅供参考）</div>
          <div class="whitespace-pre-line">{{ aiAnalysisText }}</div>
        </div>

        <!-- 填空覆写 / 简答手动标记 -->
        <div v-if="isFill || isEssay" class="flex items-center gap-2 pt-1">
          <span class="text-xs text-gray-500">
            {{ isFill ? "机器判分有误？覆写对错：" : "手动标记对错：" }}
          </span>
          <button
            class="rounded border border-green-300 px-3 py-1.5 text-xs text-green-600 hover:bg-green-50"
            :class="overriddenResult === 1 ? 'bg-green-50 font-medium' : ''"
            @click="markManual(1)"
          >
            标记为对
          </button>
          <button
            class="rounded border border-red-300 px-3 py-1.5 text-xs text-red-600 hover:bg-red-50"
            :class="overriddenResult === 0 ? 'bg-red-50 font-medium' : ''"
            @click="markManual(0)"
          >
            标记为错
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
