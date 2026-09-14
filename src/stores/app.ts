import { defineStore } from "pinia";
import { ref } from "vue";
import type { PracticeSession } from "../types";

/** 顶级视图：移动端 3 Tab（刷题为首页） */
export type TopViewKey = "practice" | "bank" | "mine";

/** 「我的」内部子视图 */
export type MineViewKey = "fault" | "collect" | "stats" | "backup" | "settings";

/** 刷题来源：从题库集、错题本或收藏发起刷题 */
export interface PracticeSource {
  kind: "bank" | "fault" | "collect";
  bankId?: number | null;
  tagId?: number | null;
}

/** 全局提示消息 */
export interface Toast {
  id: number;
  message: string;
  type: "error" | "info";
}

let toastSeq = 0;

/** 主题模式 */
export type ThemeMode = "system" | "light" | "dark";

/**
 * 全局应用状态（技术文档 3：Pinia 维护内存状态）。
 */
export const useAppStore = defineStore("app", () => {
  const currentView = ref<TopViewKey>("practice");
  const mineView = ref<MineViewKey>("fault");

  /** 当前刷题会话进度（持久化在 Rust 层 practice_session 表） */
  const session = ref<PracticeSession | null>(null);

  /** 跨视图跳转：错题本/收藏页发起刷题时设置来源后切换到刷题视图 */
  const practiceSource = ref<PracticeSource | null>(null);

  /** 全局提示消息队列 */
  const toasts = ref<Toast[]>([]);

  /** 主题模式（system / light / dark） */
  const themeMode = ref<ThemeMode>("system");
  /** 实际是否深色（由 themeMode + 系统偏好解析） */
  const isDark = ref(false);
  /** 刷题沉浸模式：移动端做题时隐藏导航（仅刷题会话态，配置态不隐藏） */
  const immersiveMode = ref(false);

  function setView(view: TopViewKey) {
    currentView.value = view;
  }

  /** 进入「我的」并定位子视图 */
  function navigateMine(view: MineViewKey) {
    mineView.value = view;
    currentView.value = "mine";
  }

  /** 从指定来源发起刷题并跳转到刷题页 */
  function startPractice(source: PracticeSource) {
    practiceSource.value = source;
    currentView.value = "practice";
  }

  /** 推送全局提示（5 秒后自动消失） */
  function pushToast(message: string, type: "error" | "info" = "error") {
    const id = ++toastSeq;
    toasts.value.push({ id, message, type });
    setTimeout(() => {
      toasts.value = toasts.value.filter((t) => t.id !== id);
    }, 5000);
  }

  function removeToast(id: number) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }

  /** 应用主题到 <html> 的 dark class */
  function applyTheme(mode: ThemeMode, systemDark: boolean) {
    themeMode.value = mode;
    isDark.value = mode === "dark" || (mode === "system" && systemDark);
    document.documentElement.classList.toggle("dark", isDark.value);
  }

  return {
    currentView,
    mineView,
    session,
    practiceSource,
    toasts,
    themeMode,
    isDark,
    immersiveMode,
    setView,
    navigateMine,
    startPractice,
    pushToast,
    removeToast,
    applyTheme,
  };
});
