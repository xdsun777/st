<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import {
  clearPracticeSession,
  getBanks,
  getCollectQuestions,
  getFaultQuestions,
  getQuestion,
  getQuestions,
  loadPracticeSession,
  savePracticeSession,
  submitAnswer,
  updateManualResult,
} from "../api";
import type { PracticeMode, PracticeSession, Question, QuestionBank } from "../types";
import { useAppStore } from "../stores/app";
import TagFilter from "../components/TagFilter.vue";
import QuestionCard from "../components/QuestionCard.vue";

/** 会话筛选信息（存于 practice_session.tag_filter 的 JSON） */
interface SessionFilter {
  kind: "bank" | "fault" | "collect";
  bank_id: number | null;
  tag_id: number | null;
  question_ids: number[];
}

type Mode = "setup" | "practice";

const store = useAppStore();

const mode = ref<Mode>("setup");
const banks = ref<QuestionBank[]>([]);
const questions = ref<Question[]>([]);
const currentIndex = ref(0);
const practiceMode = ref<PracticeMode>("order");
const source = ref<SessionFilter | null>(null);
const busy = ref(false);
const errorMsg = ref("");
const pendingRestore = ref<{ session: PracticeSession; filter: SessionFilter } | null>(null);

// setup 表单
const formBankId = ref<number | null>(null);
const formTagId = ref<number | null>(null);

const currentQuestion = computed(() => questions.value[currentIndex.value] ?? null);
const isFirst = computed(() => currentIndex.value <= 0);
const isLast = computed(() => currentIndex.value >= questions.value.length - 1);

async function loadBanks() {
  try {
    banks.value = await getBanks();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function loadQuestionsByFilter(filter: SessionFilter): Promise<Question[]> {
  if (filter.kind === "bank") {
    return getQuestions(filter.bank_id, filter.tag_id);
  }
  if (filter.kind === "fault") {
    return getFaultQuestions(filter.tag_id);
  }
  return getCollectQuestions(filter.tag_id);
}

function shuffle<T>(list: T[]): T[] {
  const result = [...list];
  for (let i = result.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

/** 保存会话：题集 id 列表 + 来源信息序列化到 tag_filter */
async function saveSession() {
  if (!source.value) return;
  const filterJson = JSON.stringify({
    kind: source.value.kind,
    bank_id: source.value.bank_id,
    tag_id: source.value.tag_id,
    question_ids: questions.value.map((q) => q.id),
  });
  try {
    await savePracticeSession(
      source.value.kind === "bank" ? source.value.bank_id ?? null : null,
      filterJson,
      currentIndex.value,
      practiceMode.value,
    );
  } catch (e) {
    errorMsg.value = String(e);
  }
}

/** 开始刷题（来源来自 setup 表单或错题/收藏页跳转） */
async function startPractice(filter: SessionFilter) {
  busy.value = true;
  errorMsg.value = "";
  try {
    let list = await loadQuestionsByFilter(filter);
    if (list.length === 0) {
      errorMsg.value =
        filter.kind === "bank" ? "该筛选条件下暂无题目" : filter.kind === "fault" ? "错题本为空" : "暂无收藏题目";
      return;
    }
    if (practiceMode.value === "random") {
      list = shuffle(list);
    }
    source.value = filter;
    questions.value = list;
    currentIndex.value = 0;
    mode.value = "practice";
    await saveSession();
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    busy.value = false;
  }
}

/** 从 setup 表单发起刷题 */
async function handleStart() {
  await startPractice({
    kind: "bank",
    bank_id: formBankId.value,
    tag_id: formTagId.value,
    question_ids: [],
  });
}

/** 继续上次会话：按保存的题目 id 顺序恢复 */
async function resumePractice() {
  if (!pendingRestore.value) return;
  const { session, filter } = pendingRestore.value;
  busy.value = true;
  errorMsg.value = "";
  try {
    const restored: Question[] = [];
    for (const id of filter.question_ids ?? []) {
      try {
        restored.push(await getQuestion(id));
      } catch {
        // 题目已删除，跳过
      }
    }
    if (restored.length === 0) {
      await clearPracticeSession();
      pendingRestore.value = null;
      errorMsg.value = "上次会话中的题目已不存在，请重新开始";
      return;
    }
    questions.value = restored;
    source.value = {
      kind: filter.kind ?? "bank",
      bank_id: filter.bank_id ?? null,
      tag_id: filter.tag_id ?? null,
      question_ids: restored.map((q) => q.id),
    };
    practiceMode.value = (session.practice_mode as PracticeMode) ?? "order";
    currentIndex.value = Math.max(0, Math.min(session.current_index, restored.length - 1));
    mode.value = "practice";
    pendingRestore.value = null;
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    busy.value = false;
  }
}

/** 放弃上次会话，重新配置 */
async function discardSession() {
  pendingRestore.value = null;
  try {
    await clearPracticeSession();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

/** 退出刷题（保留会话进度） */
function exitPractice() {
  mode.value = "setup";
  loadBanks();
}

async function handlePrev() {
  if (isFirst.value) return;
  currentIndex.value -= 1;
  await saveSession();
}

async function handleNext() {
  if (isLast.value) return;
  currentIndex.value += 1;
  await saveSession();
}

/** 提交作答：写入做题记录（essay 无机器判分） */
async function onSubmitted(payload: { userAnswer: string | string[]; machineResult: number | null }) {
  const q = currentQuestion.value;
  if (!q) return;
  try {
    await submitAnswer({
      question_id: q.id,
      user_answer: Array.isArray(payload.userAnswer)
        ? payload.userAnswer.join(",")
        : payload.userAnswer,
      machine_result: payload.machineResult,
      manual_result: null,
    });
    await saveSession();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

/** 填空覆写 / 简答手动标记对错 */
async function onOverride(result: number) {
  const q = currentQuestion.value;
  if (!q) return;
  try {
    await updateManualResult(q.id, result);
    await saveSession();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

onMounted(async () => {
  // 错题本/收藏页跳转来的刷题请求：直接开始（作废旧会话）
  const jumpSource = store.practiceSource;
  if (jumpSource) {
    store.practiceSource = null;
    await clearPracticeSession();
    practiceMode.value = "order";
    await startPractice({
      kind: jumpSource.kind,
      bank_id: jumpSource.bankId ?? null,
      tag_id: jumpSource.tagId ?? null,
      question_ids: [],
    });
    return;
  }

  await loadBanks();
  try {
    const session = await loadPracticeSession();
    if (session) {
      const filter = JSON.parse(session.tag_filter ?? "{}") as SessionFilter;
      pendingRestore.value = { session, filter };
    }
  } catch {
    // 会话解析失败则清除
    await clearPracticeSession();
  }
});

onBeforeUnmount(() => {
  if (mode.value === "practice") {
    void saveSession();
  }
});
</script>

<template>
  <section>
    <!-- 配置页 -->
    <template v-if="mode === 'setup'">
      <h2 class="text-xl font-semibold">刷题</h2>
      <p class="mb-4 mt-1 text-sm text-gray-500">
        选择题库集与标签，支持顺序 / 随机模式；中途退出自动保存进度。
      </p>

      <!-- 会话恢复提示 -->
      <div
        v-if="pendingRestore"
        class="mb-4 flex flex-col gap-3 rounded-xl border border-blue-200 bg-blue-50 p-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div class="text-sm text-blue-700">
          检测到上次未完成的刷题会话：第
          <span class="font-medium">{{ pendingRestore.session.current_index + 1 }}</span>
          题（{{ pendingRestore.filter.question_ids?.length ?? 0 }} 道题，{{
            pendingRestore.session.practice_mode === "random" ? "随机" : "顺序"
          }}模式）
        </div>
        <div class="flex gap-2">
          <button
            class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
            @click="resumePractice"
          >
            继续刷题
          </button>
          <button
            class="rounded-lg border border-gray-300 px-4 py-2 text-sm text-gray-600 hover:bg-gray-50"
            @click="discardSession"
          >
            重新开始
          </button>
        </div>
      </div>

      <div class="space-y-4 rounded-xl border border-gray-200 bg-white p-5">
        <div>
          <label class="mb-1 block text-xs text-gray-500">题库集（不选则刷全部）</label>
          <select
            v-model="formBankId"
            class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm outline-none focus:border-blue-400 sm:w-72"
          >
            <option :value="null">全部题库集</option>
            <option v-for="bank in banks" :key="bank.id" :value="bank.id">
              {{ bank.name }}（{{ bank.question_count }} 题）
            </option>
          </select>
        </div>

        <div>
          <label class="mb-1 block text-xs text-gray-500">标签筛选</label>
          <TagFilter v-model="formTagId" />
        </div>

        <div>
          <label class="mb-1 block text-xs text-gray-500">刷题模式</label>
          <div class="flex gap-3">
            <button
              v-for="option in [
                { value: 'order', label: '顺序刷题' },
                { value: 'random', label: '随机刷题' },
              ]"
              :key="option.value"
              class="rounded-lg border px-4 py-2 text-sm transition"
              :class="
                practiceMode === option.value
                  ? 'border-blue-400 bg-blue-50 text-blue-700'
                  : 'border-gray-200 text-gray-600 hover:border-gray-300'
              "
              @click="practiceMode = option.value as PracticeMode"
            >
              {{ option.label }}
            </button>
          </div>
        </div>

        <p v-if="errorMsg" class="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {{ errorMsg }}
        </p>

        <button
          class="rounded-lg bg-blue-600 px-6 py-2.5 text-sm font-medium text-white transition hover:bg-blue-700 disabled:opacity-50"
          :disabled="busy"
          @click="handleStart"
        >
          {{ busy ? "加载中…" : "开始刷题" }}
        </button>
      </div>
    </template>

    <!-- 刷题页 -->
    <template v-else>
      <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
        <div class="text-sm text-gray-500">
          {{ practiceMode === "random" ? "随机刷题" : "顺序刷题" }}
          <span v-if="source?.kind === 'fault'">· 错题本</span>
          <span v-else-if="source?.kind === 'collect'">· 收藏</span>
          <span v-else-if="source?.bank_id !== null" class="text-gray-400">
            · {{ banks.find((b) => b.id === source?.bank_id)?.name ?? "" }}
          </span>
        </div>
        <div class="flex gap-2">
          <button
            class="rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-600 transition hover:bg-gray-50 disabled:opacity-40"
            :disabled="isFirst"
            @click="handlePrev"
          >
            上一题
          </button>
          <button
            class="rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-600 transition hover:bg-gray-50 disabled:opacity-40"
            :disabled="isLast"
            @click="handleNext"
          >
            下一题
          </button>
          <button
            class="rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-500 transition hover:bg-gray-50"
            @click="exitPractice"
          >
            退出刷题
          </button>
        </div>
      </div>

      <p v-if="errorMsg" class="mb-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
        {{ errorMsg }}
      </p>

      <QuestionCard
        v-if="currentQuestion"
        :key="currentQuestion.id"
        :question="currentQuestion"
        :index="currentIndex"
        :total="questions.length"
        @submitted="onSubmitted"
        @override="onOverride"
      />
    </template>
  </section>
</template>
