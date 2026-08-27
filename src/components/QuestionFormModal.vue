<script setup lang="ts">
import { onMounted, reactive, ref, watch } from "vue";
import { createQuestion, listTags, updateQuestion } from "../api";
import type { Question, QuestionInput, QuestionType, Tag } from "../types";

/**
 * 单题新增/编辑弹窗（业务文档 4.2 / 5.1 单题 CRUD）。
 * 选项以「一行一个」编辑，提交时转为 JSON 字符串存储；标签支持已有标签点选 + 逗号分隔输入。
 */
const props = defineProps<{
  open: boolean;
  bankId: number;
  question: Question | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "saved"): void;
}>();

const TYPE_OPTIONS: { value: QuestionType; label: string }[] = [
  { value: "single", label: "单选 single" },
  { value: "multi", label: "多选 multi" },
  { value: "judge", label: "判断 judge" },
  { value: "essay", label: "简答 essay" },
  { value: "fill", label: "填空 fill" },
];

const allTags = ref<Tag[]>([]);
const errorMsg = ref("");
const saving = ref(false);

const form = reactive({
  q_type: "single" as QuestionType,
  content: "",
  optionsText: "",
  answer: "",
  analysis: "",
  tagsText: "",
});

/** 已点选的已有标签名集合 */
const selectedTagNames = ref<string[]>([]);

async function loadTags() {
  try {
    allTags.value = await listTags();
  } catch {
    allTags.value = [];
  }
}

function optionsToText(options: string | null): string {
  if (!options) return "";
  try {
    const parsed = JSON.parse(options);
    if (Array.isArray(parsed)) return parsed.join("\n");
  } catch {
    // 非 JSON 则原样展示
    return options;
  }
  return "";
}

function textToOptions(text: string): string | null {
  const parts = text
    .split("\n")
    .map((s) => s.trim())
    .filter(Boolean);
  if (parts.length === 0) return null;
  return JSON.stringify(parts);
}

function collectTags(): string[] {
  const set = new Set<string>();
  for (const t of selectedTagNames.value) set.add(t.trim());
  for (const t of form.tagsText.split(",")) {
    const name = t.trim();
    if (name) set.add(name);
  }
  return [...set];
}

function resetForm() {
  form.q_type = props.question?.q_type ?? "single";
  form.content = props.question?.content ?? "";
  form.optionsText = optionsToText(props.question?.options ?? null);
  form.answer = props.question?.answer ?? "";
  form.analysis = props.question?.analysis ?? "";
  form.tagsText = "";
  selectedTagNames.value = props.question?.tags ?? [];
  errorMsg.value = "";
}

async function handleSubmit() {
  if (!form.content.trim()) {
    errorMsg.value = "题干不能为空";
    return;
  }
  if (!form.answer.trim()) {
    errorMsg.value = "答案不能为空";
    return;
  }
  const needsOptions = form.q_type === "single" || form.q_type === "multi";
  const options = needsOptions ? textToOptions(form.optionsText) : null;

  const input: QuestionInput = {
    bank_id: props.bankId,
    q_type: form.q_type,
    content: form.content.trim(),
    options,
    answer: form.answer.trim(),
    analysis: form.analysis.trim() ? form.analysis.trim() : null,
    tags: collectTags(),
  };

  saving.value = true;
  errorMsg.value = "";
  try {
    if (props.question) {
      await updateQuestion(props.question.id, input);
    } else {
      await createQuestion(input);
    }
    emit("saved");
    emit("close");
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    saving.value = false;
  }
}

function toggleTag(name: string) {
  const index = selectedTagNames.value.indexOf(name);
  if (index >= 0) {
    selectedTagNames.value.splice(index, 1);
  } else {
    selectedTagNames.value.push(name);
  }
}

watch(
  () => props.open,
  (open) => {
    if (open) {
      resetForm();
      loadTags();
    }
  },
);

onMounted(loadTags);
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
    @click.self="emit('close')"
  >
    <div class="flex max-h-[85vh] w-full max-w-lg flex-col rounded-xl bg-white shadow-xl">
      <div class="flex items-center justify-between border-b border-gray-100 px-5 py-4">
        <h3 class="text-base font-semibold">{{ question ? "编辑题目" : "新增题目" }}</h3>
        <button class="text-gray-400 hover:text-gray-600" @click="emit('close')">关闭</button>
      </div>

      <div class="flex-1 space-y-3 overflow-y-auto px-5 py-4">
        <div>
          <label class="mb-1 block text-xs text-gray-500">题型</label>
          <select
            v-model="form.q_type"
            class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm outline-none focus:border-blue-400"
          >
            <option v-for="opt in TYPE_OPTIONS" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>

        <div>
          <label class="mb-1 block text-xs text-gray-500">题干 *</label>
          <textarea
            v-model="form.content"
            rows="3"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm outline-none focus:border-blue-400"
            placeholder="题目内容"
          />
        </div>

        <div v-if="form.q_type === 'single' || form.q_type === 'multi'">
          <label class="mb-1 block text-xs text-gray-500">选项（每行一个，如 A.选项内容）</label>
          <textarea
            v-model="form.optionsText"
            rows="4"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm outline-none focus:border-blue-400"
            placeholder="A.选项一&#10;B.选项二"
          />
        </div>

        <div>
          <label class="mb-1 block text-xs text-gray-500">
            答案 *（填空多答案用 ; 分隔；判断填 true/false；多选如 A,B）
          </label>
          <input
            v-model="form.answer"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm outline-none focus:border-blue-400"
            placeholder="正确答案"
          />
        </div>

        <div>
          <label class="mb-1 block text-xs text-gray-500">解析（可选）</label>
          <textarea
            v-model="form.analysis"
            rows="2"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm outline-none focus:border-blue-400"
            placeholder="答案解析"
          />
        </div>

        <div>
          <label class="mb-1 block text-xs text-gray-500">标签</label>
          <div v-if="allTags.length > 0" class="mb-2 flex flex-wrap gap-1.5">
            <button
              v-for="tag in allTags"
              :key="tag.id"
              class="rounded-full border px-3 py-1 text-xs transition"
              :class="
                selectedTagNames.includes(tag.name)
                  ? 'border-blue-500 bg-blue-50 text-blue-600'
                  : 'border-gray-200 text-gray-500 hover:border-gray-300'
              "
              @click="toggleTag(tag.name)"
            >
              {{ tag.name }}
            </button>
          </div>
          <input
            v-model="form.tagsText"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm outline-none focus:border-blue-400"
            placeholder="新增标签用英文逗号分隔，如：数据库,第三章"
          />
        </div>

        <p v-if="errorMsg" class="rounded-lg bg-red-50 px-3 py-2 text-xs text-red-600">
          {{ errorMsg }}
        </p>
      </div>

      <div class="flex justify-end gap-2 border-t border-gray-100 px-5 py-3">
        <button
          class="rounded-lg border border-gray-300 px-4 py-2 text-sm text-gray-600 hover:bg-gray-50"
          @click="emit('close')"
        >
          取消
        </button>
        <button
          class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
          :disabled="saving"
          @click="handleSubmit"
        >
          {{ saving ? "保存中…" : "保存" }}
        </button>
      </div>
    </div>
  </div>
</template>
