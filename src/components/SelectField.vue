<script setup lang="ts" generic="T extends string | number | null">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

/**
 * 自定义下拉选择框：替代原生 <select>。
 * 原生 select 的下拉弹层由系统渲染，深色模式下背景仍为白色、移动端弹出系统选择器，
 * 与本应用体验不符。本组件用自定义面板渲染选项，双主题一致。
 */
interface Option {
  value: T;
  label: string;
}

const props = defineProps<{
  modelValue: T;
  options: Option[];
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: T): void;
}>();

const open = ref(false);
const root = ref<HTMLElement | null>(null);

const currentLabel = computed(() => {
  const found = props.options.find((o) => o.value === props.modelValue);
  return found?.label ?? "请选择";
});

function toggle() {
  open.value = !open.value;
}

function choose(value: T) {
  emit("update:modelValue", value);
  open.value = false;
}

function onDocClick(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) {
    open.value = false;
  }
}

onMounted(() => document.addEventListener("click", onDocClick));
onBeforeUnmount(() => document.removeEventListener("click", onDocClick));
</script>

<template>
  <div ref="root" class="relative">
    <button
      type="button"
      class="flex w-full items-center justify-between gap-2 rounded-lg border border-gray-300 bg-white px-3 py-2 text-left text-sm outline-none transition focus:border-blue-400"
      @click="toggle"
    >
      <span class="min-w-0 flex-1 truncate text-gray-700">{{ currentLabel }}</span>
      <svg
        class="h-4 w-4 shrink-0 text-gray-400 transition-transform"
        :class="open ? 'rotate-180' : ''"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
      </svg>
    </button>

    <div
      v-if="open"
      class="absolute left-0 right-0 z-20 mt-1 max-h-60 overflow-y-auto rounded-lg border border-gray-200 bg-white py-1 shadow-lg"
    >
      <button
        v-for="opt in options"
        :key="String(opt.value)"
        type="button"
        class="flex w-full items-center justify-between gap-2 px-3 py-2 text-left text-sm transition hover:bg-gray-100"
        @click="choose(opt.value)"
      >
        <span
          class="min-w-0 flex-1 truncate"
          :class="opt.value === modelValue ? 'text-blue-600' : 'text-gray-700'"
        >
          {{ opt.label }}
        </span>
        <svg
          v-if="opt.value === modelValue"
          class="h-4 w-4 shrink-0 text-blue-600"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M4.5 12.75l6 6 9-13.5" />
        </svg>
      </button>
    </div>
  </div>
</template>
