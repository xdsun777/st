<script setup lang="ts">
defineProps<{
  open: boolean;
  title: string;
  message: string;
  confirmText?: string;
  danger?: boolean;
}>();

const emit = defineEmits<{
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
    @click.self="emit('cancel')"
  >
    <div class="w-full max-w-sm rounded-xl bg-white p-5 shadow-xl">
      <h3 class="text-base font-semibold">{{ title }}</h3>
      <p class="mt-2 whitespace-pre-line text-sm text-gray-600">{{ message }}</p>
      <div class="mt-5 flex justify-end gap-2">
        <button
          class="rounded-lg border border-gray-300 px-4 py-2 text-sm text-gray-600 transition hover:bg-gray-50"
          @click="emit('cancel')"
        >
          取消
        </button>
        <button
          class="rounded-lg px-4 py-2 text-sm font-medium text-white transition"
          :class="danger ? 'bg-red-600 hover:bg-red-700' : 'bg-blue-600 hover:bg-blue-700'"
          @click="emit('confirm')"
        >
          {{ confirmText ?? "确认" }}
        </button>
      </div>
    </div>
  </div>
</template>
