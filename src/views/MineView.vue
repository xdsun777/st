<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { useAppStore, type MineViewKey } from "../stores/app";
import FaultView from "./FaultView.vue";
import CollectView from "./CollectView.vue";
import StatsView from "./StatsView.vue";
import BackupView from "./BackupView.vue";
import SettingsView from "./SettingsView.vue";
import AboutView from "./AboutView.vue";
import { getCollectQuestions, getFaultQuestions } from "../api";

/**
 * 「我的」收纳页：
 * - 桌面端：侧边栏已直达子视图，直接渲染对应子视图
 * - 移动端：先显示列表导航（带角标），点击进入子视图，返回键回列表
 */
const store = useAppStore();

const isDesktop = ref(window.innerWidth >= 768);
const subOpen = ref(false);
const faultCount = ref(0);
const collectCount = ref(0);

const items: { key: MineViewKey; label: string; desc: string; icon: string[]; badge?: () => number }[] = [
  {
    key: "fault",
    label: "错题本",
    desc: "答错的题自动归集",
    icon: ["M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z"],
    badge: () => faultCount.value,
  },
  {
    key: "collect",
    label: "收藏",
    desc: "手动收藏的题",
    icon: [
      "M11.48 3.499a.562.562 0 011.04 0l2.125 5.111a.563.563 0 00.475.345l5.518.442c.499.04.701.663.321.988l-4.204 3.602a.563.563 0 00-.182.557l1.285 5.385a.562.562 0 01-.84.61l-4.725-2.885a.563.563 0 00-.586 0L6.982 20.54a.562.562 0 01-.84-.61l1.285-5.386a.562.562 0 00-.182-.557l-4.204-3.602a.563.563 0 01.321-.988l5.518-.442a.563.563 0 00.475-.345L11.48 3.5z",
    ],
    badge: () => collectCount.value,
  },
  {
    key: "stats",
    label: "统计",
    desc: "正确率与刷题量",
    icon: [
      "M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z",
    ],
  },
  {
    key: "backup",
    label: "备份恢复",
    desc: "数据导出 / 恢复",
    icon: ["M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3"],
  },
  {
    key: "settings",
    label: "设置",
    desc: "主题 / AI 配置",
    icon: [
      "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z",
      "M15 12a3 3 0 11-6 0 3 3 0 016 0z",
    ],
  },
  {
    key: "about",
    label: "关于",
    desc: "版本与开源信息",
    icon: ["M11.25 11.25l.041-.02a.75.75 0 011.063.852l-.708 2.836a.75.75 0 001.063.853l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z"],
  },
];

const viewMap: Record<MineViewKey, unknown> = {
  fault: FaultView,
  collect: CollectView,
  stats: StatsView,
  backup: BackupView,
  settings: SettingsView,
  about: AboutView,
};

function openItem(key: MineViewKey) {
  store.navigateMine(key);
  subOpen.value = true;
}

function backToList() {
  subOpen.value = false;
}

function onResize() {
  isDesktop.value = window.innerWidth >= 768;
}

async function loadBadges() {
  try {
    faultCount.value = (await getFaultQuestions(null)).length;
    collectCount.value = (await getCollectQuestions(null)).length;
  } catch {
    // 角标加载失败不影响页面
  }
}

onMounted(() => {
  window.addEventListener("resize", onResize);
  loadBadges();
});

onBeforeUnmount(() => window.removeEventListener("resize", onResize));
</script>

<template>
  <section>
    <!-- 桌面端：直接渲染子视图 -->
    <template v-if="isDesktop">
      <component :is="viewMap[store.mineView]" />
    </template>

    <!-- 移动端：列表页 / 子视图 -->
    <template v-else>
      <div v-if="!subOpen">
        <h2 class="text-xl font-semibold">我的</h2>
        <p class="mb-4 mt-1 text-sm text-gray-500">错题本、收藏、统计与设置。</p>
        <ul class="space-y-2">
          <li v-for="item in items" :key="item.key">
            <button
              class="flex w-full items-center gap-3 rounded-xl border border-gray-200 bg-white p-4 text-left transition hover:border-gray-300"
              @click="openItem(item.key)"
            >
              <svg
                class="h-6 w-6 shrink-0 text-gray-400"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path v-for="d in item.icon" :key="d" :d="d" />
              </svg>
              <div class="min-w-0 flex-1">
                <div class="text-sm font-medium">{{ item.label }}</div>
                <div class="mt-0.5 text-xs text-gray-400">{{ item.desc }}</div>
              </div>
              <span
                v-if="item.badge && item.badge() > 0"
                class="rounded-full bg-blue-50 px-2 py-0.5 text-xs text-blue-600"
              >
                {{ item.badge() }}
              </span>
              <span class="text-gray-300">›</span>
            </button>
          </li>
        </ul>
      </div>

      <div v-else>
        <button class="mb-3 text-sm text-blue-600" @click="backToList">‹ 返回我的</button>
        <component :is="viewMap[store.mineView]" />
      </div>
    </template>
  </section>
</template>
