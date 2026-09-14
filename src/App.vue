<script setup lang="ts">
import { computed } from "vue";
import { useAppStore, type MineViewKey, type TopViewKey } from "./stores/app";
import BankView from "./views/BankView.vue";
import PracticeView from "./views/PracticeView.vue";
import MineView from "./views/MineView.vue";
import TitleBar from "./components/TitleBar.vue";

const app = useAppStore();

interface NavItem {
  label: string;
  short: string;
  desc: string;
  top?: TopViewKey;
  mine?: MineViewKey;
}

const navGroups: { key: string; items: NavItem[] }[] = [
  {
    key: "main",
    items: [
      { top: "practice", label: "刷题", short: "刷题", desc: "顺序 / 随机刷题" },
      { top: "bank", label: "题库", short: "题库", desc: "题库集 / 题目 / 标签 / 导入" },
    ],
  },
  {
    key: "data",
    items: [
      { mine: "fault", label: "错题本", short: "错题", desc: "错题复习" },
      { mine: "collect", label: "收藏", short: "收藏", desc: "收藏题目" },
      { mine: "stats", label: "统计", short: "统计", desc: "正确率统计" },
    ],
  },
  {
    key: "system",
    items: [
      { mine: "backup", label: "备份恢复", short: "备份", desc: "数据导出 / 恢复" },
      { mine: "settings", label: "设置", short: "设置", desc: "主题 / AI 配置" },
    ],
  },
];

/** 移动端底部 3 Tab */
const mobileTabs: { top: TopViewKey; label: string; short: string }[] = [
  { top: "practice", label: "刷题", short: "刷题" },
  { top: "bank", label: "题库", short: "题库" },
  { top: "mine", label: "我的", short: "我的" },
];

/** 内联 SVG 图标 path（heroicons outline 24x24） */
const icons: Record<string, string[]> = {
  practice: [
    "M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10",
  ],
  bank: [
    "M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25",
  ],
  mine: [
    "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z",
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
  settings: [
    "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z",
    "M15 12a3 3 0 11-6 0 3 3 0 016 0z",
  ],
};

function iconOf(item: NavItem | { top: TopViewKey }): string[] {
  const key = item.top ?? (item as NavItem).mine ?? "mine";
  return icons[key] ?? icons.mine;
}

function isItemActive(item: NavItem): boolean {
  if (item.top) return app.currentView === item.top;
  return app.currentView === "mine" && app.mineView === item.mine;
}

function onItemClick(item: NavItem) {
  if (item.top) app.setView(item.top);
  else if (item.mine) app.navigateMine(item.mine);
}

const views: Record<TopViewKey, unknown> = {
  practice: PracticeView,
  bank: BankView,
  mine: MineView,
};

const currentView = computed(() => views[app.currentView]);
</script>

<template>
  <div class="flex h-full flex-col bg-gray-50 text-gray-900">
    <!-- 桌面端自定义标题栏（移动端隐藏） -->
    <TitleBar />

    <div class="flex min-h-0 flex-1 flex-col md:flex-row md:overflow-hidden">
    <!-- 移动端顶部标题栏（刷题沉浸模式隐藏） -->
    <header
      class="shrink-0 border-b border-gray-200 bg-white px-4 pb-2 pt-3 md:hidden"
      :class="{ hidden: app.immersiveMode }"
      style="padding-top: max(0.75rem, env(safe-area-inset-top))"
    >
      <h1 class="text-base font-semibold">再刷一题</h1>
      <p class="text-xs text-gray-400">每日一语</p>
    </header>

    <!-- 桌面端侧边栏（分组） -->
    <aside class="hidden w-56 shrink-0 flex-col border-r border-gray-200 bg-white md:flex">
      <div class="border-b border-gray-100 px-5 py-4">
        <h1 class="text-lg font-semibold">再刷一题</h1>
        <p class="mt-0.5 text-xs text-gray-400">每日一语</p>
      </div>
      <nav class="flex-1 space-y-4 overflow-y-auto p-3">
        <div v-for="group in navGroups" :key="group.key">
          <div class="mb-1 px-3 text-[11px] font-medium uppercase tracking-wide text-gray-400">
            {{ group.key === "main" ? "主功能" : group.key === "data" ? "复习与数据" : "系统" }}
          </div>
          <div class="space-y-1">
            <button
              v-for="item in group.items"
              :key="item.label"
              class="flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left transition"
              :class="
                isItemActive(item)
                  ? 'bg-blue-50 text-blue-700'
                  : 'text-gray-600 hover:bg-gray-100'
              "
              @click="onItemClick(item)"
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
                <path v-for="d in iconOf(item)" :key="d" :d="d" />
              </svg>
              <div>
                <div class="text-sm" :class="isItemActive(item) ? 'font-medium' : ''">
                  {{ item.label }}
                </div>
                <div
                  class="mt-0.5 text-xs"
                  :class="isItemActive(item) ? 'text-blue-400' : 'text-gray-400'"
                >
                  {{ item.desc }}
                </div>
              </div>
            </button>
          </div>
        </div>
      </nav>
    </aside>

    <!-- 内容区：移动端为底部导航预留空间 -->
    <main
      class="flex-1 overflow-y-auto p-4 pb-24 sm:p-6 md:pb-6"
      :class="{ 'p-0 pb-0': app.immersiveMode }"
      :style="
        app.immersiveMode
          ? { paddingTop: 'env(safe-area-inset-top)', paddingBottom: 'env(safe-area-inset-bottom)' }
          : undefined
      "
    >
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

    <!-- 移动端底部 Tab 导航（3 项；刷题沉浸模式隐藏） -->
    <nav
      class="fixed inset-x-0 bottom-0 z-10 border-t border-gray-200 bg-white md:hidden"
      :class="{ hidden: app.immersiveMode }"
      style="padding-bottom: env(safe-area-inset-bottom)"
    >
      <div class="grid grid-cols-3">
        <button
          v-for="tab in mobileTabs"
          :key="tab.top"
          class="flex min-h-[3.5rem] flex-col items-center justify-center gap-0.5"
          :class="app.currentView === tab.top ? 'text-blue-600' : 'text-gray-500'"
          @click="app.setView(tab.top)"
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
            <path v-for="d in icons[tab.top]" :key="d" :d="d" />
          </svg>
          <span
            class="text-[11px] leading-none"
            :class="app.currentView === tab.top ? 'font-medium' : ''"
          >
            {{ tab.short }}
          </span>
        </button>
      </div>
    </nav>
  </div>
</template>
