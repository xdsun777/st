<script setup lang="ts">
import { onMounted, ref } from "vue";
import { listTags } from "../api";
import type { Tag } from "../types";

/** 标签筛选组件：单选，null 表示「全部」 */
const props = defineProps<{
  modelValue: number | null;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: number | null): void;
}>();

const tags = ref<Tag[]>([]);

async function refresh() {
  try {
    tags.value = await listTags();
  } catch {
    tags.value = [];
  }
}

onMounted(refresh);

defineExpose({ refresh });
</script>

<template>
  <div class="flex flex-wrap items-center gap-1.5">
    <button
      class="rounded-full border px-3 py-1 text-xs transition"
      :class="
        props.modelValue === null
          ? 'border-blue-500 bg-blue-50 text-blue-600'
          : 'border-gray-200 bg-white text-gray-500 hover:border-gray-300'
      "
      @click="emit('update:modelValue', null)"
    >
      全部
    </button>
    <button
      v-for="tag in tags"
      :key="tag.id"
      class="rounded-full border px-3 py-1 text-xs transition"
      :class="
        props.modelValue === tag.id
          ? 'border-blue-500 bg-blue-50 text-blue-600'
          : 'border-gray-200 bg-white text-gray-500 hover:border-gray-300'
      "
      @click="emit('update:modelValue', tag.id)"
    >
      {{ tag.name }}
    </button>
    <span v-if="tags.length === 0" class="text-xs text-gray-400">暂无标签</span>
  </div>
</template>
