<script setup lang="ts">
import { computed } from "vue";
import { useAppStore, type ViewKey } from "./stores/app";
import BankView from "./views/BankView.vue";
import PracticeView from "./views/PracticeView.vue";
import FaultView from "./views/FaultView.vue";
import CollectView from "./views/CollectView.vue";
import StatsView from "./views/StatsView.vue";
import BackupView from "./views/BackupView.vue";
import TitleBar from "./components/TitleBar.vue";

const app = useAppStore();

interface NavItem {
  key: ViewKey;
  label: string;
  short: string;
  desc: string;
}

const navItems: NavItem[] = [
  { key: "bank", label: "题库管理", short: "题库", desc: "题库集 / 题目 / 标签 / CSV导入" },
  { key: "practice", label: "刷题", short: "刷题", desc: "顺序 / 随机刷题" },
  { key: "fault", label: "错题本", short: "错题", desc: "错题复习" },
  { key: "collect", label: "收藏", short: "收藏", desc: "收藏题目" },
  { key: "stats", label: "统计", short: "统计", desc: "正确率统计" },
  { key: "backup", label: "备份恢复", short: "备份", desc: "数据导出 / 恢复" },
];

/** 内联 SVG 图标 path（heroicons outline 24x24），避免引入额外图标依赖 */
const icons: Record<ViewKey, string[]> = {
  bank: [
    "M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25",
  ],
  practice: [
    "M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10",
  ],
  fault: ["M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z"],
  collect: [
    "M11.48 3.499a.562.562 0 011.04 0l2.125 5.111a.563.563 0 00.475.345l5.518.442c.499.04.701.663.321.988l-4.204 3.602a.563.563 0 00-.182.557l1.285 5.385a.562.562 0 01-.84.61l-4.725-2.885a.563.563 0 00-.586 0L6.982 20.54a.562.562 0 01-.84-.61l1.285-5.386a.562.562 0 00-.182-.557l-4.204-3.602a.563.563 0 01.321-.988l5.518-.442a.563.563 0 00.475-.345L11.48 3.5z",
  ],
  stats: [
    "M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z",
  ],
  backup: [
    "M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3",
  ],
};

const views: Record<ViewKey, unknown> = {
  bank: BankView,
  practice: PracticeView,
  fault: FaultView,
  collect: CollectView,
  stats: StatsView,
  backup: BackupView,
};

const currentView = computed(() => views[app.currentView]);
</script>

<template>
  <div class="flex h-full flex-col bg-gray-50 text-gray-900">
    <!-- 桌面端自定义标题栏（移动端隐藏） -->
    <TitleBar />

    <div class="flex min-h-0 flex-1 flex-col md:flex-row md:overflow-hidden">
    <!-- 移动端顶部标题栏 -->
    <header
      class="shrink-0 border-b border-gray-200 bg-white px-4 pb-2 pt-3 md:hidden"
      style="padding-top: max(0.75rem, env(safe-area-inset-top))"
    >
      <h1 class="text-base font-semibold">再刷一题</h1>
      <p class="text-xs text-gray-400">每日一语</p>
    </header>

    <!-- 桌面端侧边栏 -->
    <aside class="hidden w-56 shrink-0 flex-col border-r border-gray-200 bg-white md:flex">
      <div class="border-b border-gray-100 px-5 py-4">
        <h1 class="text-lg font-semibold">再刷一题</h1>
        <p class="mt-0.5 text-xs text-gray-400">每日一语</p>
      </div>
      <nav class="flex-1 space-y-1 p-3">
        <button
          v-for="item in navItems"
          :key="item.key"
          class="flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left transition"
          :class="
            app.currentView === item.key
              ? 'bg-blue-50 text-blue-700'
              : 'text-gray-600 hover:bg-gray-100'
          "
          @click="app.setView(item.key)"
        >
          <svg
            class="h-5 w-5 shrink-0"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path v-for="d in icons[item.key]" :key="d" :d="d" />
          </svg>
          <div>
            <div class="text-sm" :class="app.currentView === item.key ? 'font-medium' : ''">
              {{ item.label }}
            </div>
            <div
              class="mt-0.5 text-xs"
              :class="app.currentView === item.key ? 'text-blue-400' : 'text-gray-400'"
            >
              {{ item.desc }}
            </div>
          </div>
        </button>
      </nav>
    </aside>

    <!-- 内容区：移动端为底部导航预留空间 -->
    <main class="flex-1 overflow-y-auto p-4 pb-24 sm:p-6 md:pb-6">
      <component :is="currentView" />
    </main>
    </div>

    <!-- 全局错误/信息提示 -->
    <div class="pointer-events-none fixed inset-x-0 top-3 z-[60] flex flex-col items-center gap-2 px-4">
      <div
        v-for="toast in app.toasts"
        :key="toast.id"
        class="pointer-events-auto w-full max-w-md rounded-lg px-4 py-2.5 text-sm text-white shadow-lg"
        :class="toast.type === 'error' ? 'bg-red-600' : 'bg-gray-800'"
        @click="app.removeToast(toast.id)"
      >
        {{ toast.message }}
      </div>
    </div>

    <!-- 移动端底部 Tab 导航 -->
    <nav
      class="fixed inset-x-0 bottom-0 z-10 border-t border-gray-200 bg-white md:hidden"
      style="padding-bottom: env(safe-area-inset-bottom)"
    >
      <div class="grid grid-cols-6">
        <button
          v-for="item in navItems"
          :key="item.key"
          class="flex min-h-[3.5rem] flex-col items-center justify-center gap-0.5"
          :class="app.currentView === item.key ? 'text-blue-600' : 'text-gray-500'"
          @click="app.setView(item.key)"
        >
          <svg
            class="h-6 w-6"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path v-for="d in icons[item.key]" :key="d" :d="d" />
          </svg>
          <span
            class="text-[11px] leading-none"
            :class="app.currentView === item.key ? 'font-medium' : ''"
          >
            {{ item.short }}
          </span>
        </button>
      </div>
    </nav>
  </div>
</template>
