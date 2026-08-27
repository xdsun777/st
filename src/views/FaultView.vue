<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { getFaultQuestions, updateFault } from "../api";
import type { Question, QuestionType } from "../types";
import TagFilter from "../components/TagFilter.vue";

const TYPE_LABELS: Record<QuestionType, string> = {
  single: "单选",
  multi: "多选",
  judge: "判断",
  essay: "简答",
  fill: "填空",
};

const questions = ref<Question[]>([]);
const tagId = ref<number | null>(null);
const loading = ref(false);
const errorMsg = ref("");

async function refresh() {
  loading.value = true;
  errorMsg.value = "";
  try {
    questions.value = await getFaultQuestions(tagId.value);
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function handleRemoveFault(questionId: number) {
  errorMsg.value = "";
  try {
    // 移出错题：仅清除标记，做题记录完整保留（业务文档 4.5.3）
    await updateFault(questionId, false);
    await refresh();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

watch(tagId, () => refresh());
onMounted(refresh);
</script>

<template>
  <section>
    <h2 class="text-xl font-semibold">错题本</h2>
    <p class="mb-3 mt-1 text-sm text-gray-500">
      答错的题目会自动进入这里；移出错题仅清除标记，做题记录保留。
    </p>

    <div class="mb-3"><TagFilter v-model="tagId" /></div>
    <p v-if="errorMsg" class="mb-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
      {{ errorMsg }}
    </p>
    <p v-if="loading" class="text-sm text-gray-400">加载中…</p>

    <div
      v-else-if="questions.length === 0"
      class="rounded-xl border-2 border-dashed border-gray-200 bg-white p-10 text-center text-sm text-gray-400"
    >
      暂无错题。
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
          <button
            class="shrink-0 rounded px-2 py-1 text-xs text-gray-500 hover:text-red-600"
            @click="handleRemoveFault(q.id)"
          >
            移出错题
          </button>
        </div>
      </li>
    </ul>
  </section>
</template>
