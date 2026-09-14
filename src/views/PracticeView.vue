<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import {
  clearPracticeSession,
  updateAiResult,
  getAiConfig,
  getBanks,
  getCollectQuestions,
  getFaultQuestions,
  getQuestion,
  getQuestions,
  getSetting,
  loadPracticeSession,
  savePracticeSession,
  setSetting,
  submitAnswer,
  updateCollect,
  updateManualResult,
} from "../api";
import type { PracticeMode, PracticeSession, Question, QuestionBank } from "../types";
import { useAppStore } from "../stores/app";
import { analyzeMistakeJs, judgeEssayJs } from "../utils/ai";
import TagFilter from "../components/TagFilter.vue";
import QuestionCard from "../components/QuestionCard.vue";
import SelectField from "../components/SelectField.vue";

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

// AI 状态（essay 判题 / 错题解析）
const aiJudgeEnabled = ref(false);
const aiAnalysisEnabled = ref(false);
const aiJudging = ref(false);
const aiJudgeResult = ref<{ correct: number; reason: string } | null>(null);
const aiAnalysisText = ref<string | null>(null);
/** 当前题提交的用户作答（手动标错时用于触发解析） */
const currentUserAnswer = ref("");

// 会话级熔断：连续失败 3 次暂停本次会话 AI 功能（业务文档 6.6）
const aiFailCount = ref(0);
const aiPaused = ref(false);

// 轻量激励
const encourageEnabled = ref(true);
const encourageMsg = ref("");
const streakDays = ref(0);
const streakCorrectCount = ref(0);
let encourageTimer: ReturnType<typeof setTimeout> | null = null;

// setup 表单
const formBankId = ref<number | null>(null);
const formTagId = ref<number | null>(null);

/** 题库集下拉选项（含「全部题库集」） */
const bankOptions = computed(() => [
  { value: null, label: "全部题库集" },
  ...banks.value.map((b) => ({ value: b.id, label: `${b.name}（${b.question_count} 题）` })),
]);

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
    store.immersiveMode = true;
    resetAiBreaker();
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
    store.immersiveMode = true;
    resetAiBreaker();
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
  store.immersiveMode = false;
  loadBanks();
}

async function handlePrev() {
  if (isFirst.value) return;
  currentIndex.value -= 1;
  resetAiState();
  await saveSession();
}

async function handleNext() {
  if (isLast.value) return;
  currentIndex.value += 1;
  resetAiState();
  await saveSession();
}

/** 提交作答：写入做题记录；essay 触发 AI 判题；判错触发 AI 解析；累计激励 */
async function onSubmitted(payload: { userAnswer: string | string[]; machineResult: number | null }) {
  const q = currentQuestion.value;
  if (!q) return;
  const userAnswer = Array.isArray(payload.userAnswer)
    ? payload.userAnswer.join(",")
    : payload.userAnswer;
  currentUserAnswer.value = userAnswer;
  try {
    const record = await submitAnswer({
      question_id: q.id,
      user_answer: userAnswer,
      machine_result: payload.machineResult,
      manual_result: null,
    });

    // essay：AI 判题（可选，熔断后跳过）
    if (q.q_type === "essay" && aiJudgeEnabled.value && !aiPaused.value) {
      aiJudging.value = true;
      try {
        aiJudgeResult.value = await judgeEssayJs({
          content: q.content,
          answer: q.answer,
          userAnswer,
        });
        recordAiSuccess();
        // 写回数据库：同步统计与错题本（失败不影响前端展示）
        updateAiResult(record.id, aiJudgeResult.value.correct).catch(() => {});
      } catch (e) {
        recordAiFailure();
        errorMsg.value = `${String(e)}（请手动标记对错）`;
      } finally {
        aiJudging.value = false;
      }
    }

    // 最终判错：AI 错题解析（可选，异步）
    const finalCorrect =
      payload.machineResult === null
        ? aiJudgeResult.value?.correct ?? null
        : payload.machineResult;
    if (finalCorrect === 0 && aiAnalysisEnabled.value && !aiPaused.value) {
      analyzeMistakeJs({
        questionId: q.id,
        content: q.content,
        answer: q.answer,
        userAnswer,
        analysis: q.analysis ?? undefined,
      })
        .then((text) => {
          aiAnalysisText.value = text;
          recordAiSuccess();
        })
        .catch(() => {
          // 静默降级，但计入熔断
          recordAiFailure();
        });
    }

    // 轻量激励：连续答对计数 + 文案
    const correct = finalCorrect === 1;
    await handleStreak(correct);

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
    // 手动标错 → 触发 AI 错题解析（业务文档 6.4：以最终结果为准）
    if (result === 0 && aiAnalysisEnabled.value && !aiPaused.value && currentUserAnswer.value) {
      analyzeMistakeJs({
        questionId: q.id,
        content: q.content,
        answer: q.answer,
        userAnswer: currentUserAnswer.value,
        analysis: q.analysis ?? undefined,
      })
        .then((text) => {
          aiAnalysisText.value = text;
          recordAiSuccess();
        })
        .catch(() => {
          recordAiFailure();
        });
    }
    await saveSession();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

/** 收藏 / 取消收藏当前题目（业务文档 4.6：收藏独立于错题） */
async function onToggleCollect() {
  const q = currentQuestion.value;
  if (!q) return;
  try {
    const target = q.is_collect === 1 ? 0 : 1;
    await updateCollect(q.id, target === 1);
    q.is_collect = target;
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
  await loadAiAndEncourageConfig();
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
  if (encourageTimer) clearTimeout(encourageTimer);
});

/** 读取 AI 开关与激励开关 */
async function loadAiAndEncourageConfig() {
  try {
    const cfg = await getAiConfig();
    aiJudgeEnabled.value = cfg.judge_enabled;
    aiAnalysisEnabled.value = cfg.analysis_enabled;
  } catch {
    // 未配置视为关闭
  }
  try {
    encourageEnabled.value = (await getSetting("encourage_enabled")) !== "0";
  } catch {
    encourageEnabled.value = true;
  }
  try {
    streakDays.value = Number((await getSetting("streak_days")) ?? 0);
  } catch {
    streakDays.value = 0;
  }
}

/** 重置当前题的 AI 状态 */
function resetAiState() {
  aiJudging.value = false;
  aiJudgeResult.value = null;
  aiAnalysisText.value = null;
}

/** 重置会话级熔断（新会话开始时调用） */
function resetAiBreaker() {
  aiFailCount.value = 0;
  aiPaused.value = false;
}

/** AI 调用失败计数；连续失败 3 次暂停本次会话 AI 功能 */
function recordAiFailure() {
  if (aiPaused.value) return;
  aiFailCount.value += 1;
  if (aiFailCount.value >= 3) {
    aiPaused.value = true;
  }
}

/** AI 调用成功，重置连续失败计数 */
function recordAiSuccess() {
  aiFailCount.value = 0;
}

const ENCOURAGE_WORDS = ["再刷一题", "状态不错", "手感来了", "继续保持", "渐入佳境"];

/** 轻量激励：连续答对计数与文案；更新本地连续刷题天数 */
async function handleStreak(correct: boolean) {
  if (correct) {
    streakCorrectCount.value += 1;
    if (encourageEnabled.value && streakCorrectCount.value % 5 === 0) {
      encourageMsg.value =
        ENCOURAGE_WORDS[Math.floor(Math.random() * ENCOURAGE_WORDS.length)];
      if (encourageTimer) clearTimeout(encourageTimer);
      encourageTimer = setTimeout(() => {
        encourageMsg.value = "";
      }, 1500);
    }
  } else {
    streakCorrectCount.value = 0;
  }
  // 更新连续刷题天数（本地）
  try {
    const today = new Date();
    const todayStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, "0")}-${String(today.getDate()).padStart(2, "0")}`;
    const last = (await getSetting("last_practice_date")) ?? "";
    if (last !== todayStr) {
      const yesterday = new Date(today.getTime() - 86400000);
      const yesterdayStr = `${yesterday.getFullYear()}-${String(yesterday.getMonth() + 1).padStart(2, "0")}-${String(yesterday.getDate()).padStart(2, "0")}`;
      const next = last === yesterdayStr ? streakDays.value + 1 : 1;
      streakDays.value = next;
      await setSetting("last_practice_date", todayStr);
      await setSetting("streak_days", String(next));
    }
  } catch {
    // 忽略连续天数写入失败
  }
}
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

      <!-- 快捷入口 + 轻量概览 -->
      <div class="mb-4 grid grid-cols-2 gap-3">
        <button
          class="rounded-xl border border-gray-200 bg-white p-4 text-left transition hover:border-blue-300"
          @click="store.startPractice({ kind: 'fault', tagId: null })"
        >
          <div class="text-sm font-medium">错题复习</div>
          <div class="mt-0.5 text-xs text-gray-400">已答错的题</div>
        </button>
        <button
          class="rounded-xl border border-gray-200 bg-white p-4 text-left transition hover:border-blue-300"
          @click="store.startPractice({ kind: 'collect', tagId: null })"
        >
          <div class="text-sm font-medium">收藏刷题</div>
          <div class="mt-0.5 text-xs text-gray-400">手动收藏的题</div>
        </button>
      </div>
      <div
        v-if="streakDays > 0"
        class="mb-4 rounded-xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-600"
      >
        已连续刷题 {{ streakDays }} 天，继续保持 ✦
      </div>

      <div class="space-y-4 rounded-xl border border-gray-200 bg-white p-5">
        <div>
          <label class="mb-1 block text-xs text-gray-500">题库集（不选则刷全部）</label>
          <SelectField v-model="formBankId" :options="bankOptions" class="sm:w-72" />
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

      <div
        v-if="aiPaused"
        class="mb-3 rounded-lg bg-amber-50 px-3 py-2 text-sm text-amber-600"
      >
        AI 服务暂不可用，已暂停本次会话
      </div>

      <!-- 轻量激励文案 -->
      <div
        v-if="encourageMsg"
        class="mb-3 text-center text-sm font-medium text-amber-600 transition-opacity"
      >
        {{ encourageMsg }}
      </div>

      <QuestionCard
        v-if="currentQuestion"
        :key="currentQuestion.id"
        :question="currentQuestion"
        :index="currentIndex"
        :total="questions.length"
        :ai-judge-result="aiJudgeResult"
        :ai-judging="aiJudging"
        :ai-analysis-text="aiAnalysisText"
        @submitted="onSubmitted"
        @override="onOverride"
        @toggle-collect="onToggleCollect"
      />
    </template>
  </section>
</template>
