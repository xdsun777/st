<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { createTag, deleteTag, listTags } from "../api";
import type { Tag } from "../types";

/**
 * 标签管理弹窗：列出、新建、删除标签。
 * 删除标签仅清除题目标记，不删除题目（业务文档 4.4）。
 */
const props = defineProps<{ open: boolean }>();
const emit = defineEmits<{
  (e: "close"): void;
  (e: "changed"): void;
}>();

const tags = ref<Tag[]>([]);
const newName = ref("");
const errorMsg = ref("");
const confirmDeleteId = ref<number | null>(null);

async function refresh() {
  try {
    tags.value = await listTags();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function handleCreate() {
  const name = newName.value.trim();
  if (!name) return;
  errorMsg.value = "";
  try {
    await createTag(name);
    newName.value = "";
    await refresh();
    emit("changed");
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function handleDelete(id: number) {
  errorMsg.value = "";
  try {
    await deleteTag(id);
    confirmDeleteId.value = null;
    await refresh();
    emit("changed");
  } catch (e) {
    errorMsg.value = String(e);
  }
}

watch(
  () => props.open,
  (open) => {
    if (open) {
      errorMsg.value = "";
      confirmDeleteId.value = null;
      refresh();
    }
  },
);

onMounted(refresh);
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
    @click.self="emit('close')"
  >
    <div class="flex max-h-[80vh] w-full max-w-md flex-col rounded-xl bg-white shadow-xl">
      <div class="flex items-center justify-between border-b border-gray-100 px-5 py-4">
        <h3 class="text-base font-semibold">标签管理</h3>
        <button class="text-gray-400 hover:text-gray-600" @click="emit('close')">✕</button>
      </div>

      <div class="flex gap-2 border-b border-gray-100 px-5 py-3">
        <input
          v-model="newName"
          class="min-w-0 flex-1 rounded-lg border border-gray-300 px-3 py-2 text-sm outline-none focus:border-blue-400"
          placeholder="新标签名"
          @keyup.enter="handleCreate"
        />
        <button
          class="shrink-0 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
          @click="handleCreate"
        >
          新建
        </button>
      </div>

      <p v-if="errorMsg" class="px-5 pt-2 text-xs text-red-600">{{ errorMsg }}</p>

      <div class="flex-1 overflow-y-auto p-3">
        <p v-if="tags.length === 0" class="p-6 text-center text-sm text-gray-400">
          暂无标签
        </p>
        <div v-else class="space-y-1">
          <div
            v-for="tag in tags"
            :key="tag.id"
            class="flex items-center justify-between rounded-lg px-3 py-2 hover:bg-gray-50"
          >
            <span class="text-sm">{{ tag.name }}</span>
            <button
              v-if="confirmDeleteId !== tag.id"
              class="rounded px-2 py-1 text-xs text-red-500 hover:text-red-600"
              @click="confirmDeleteId = tag.id"
            >
              删除
            </button>
            <div v-else class="flex items-center gap-1">
              <span class="text-xs text-gray-400">确认？</span>
              <button
                class="rounded bg-red-500 px-2 py-1 text-xs text-white hover:bg-red-600"
                @click="handleDelete(tag.id)"
              >
                删除
              </button>
              <button class="rounded px-2 py-1 text-xs text-gray-500" @click="confirmDeleteId = null">
                取消
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
