import { defineStore } from "pinia";
import { ref } from "vue";
import type { PracticeSession } from "../types";

/** 主界面视图 */
export type ViewKey = "bank" | "practice" | "fault" | "collect" | "stats" | "backup";

/**
 * 全局应用状态（技术文档 3：Pinia 维护内存状态）。
 * 当前刷题会话、筛选条件等后续在刷题模块中扩展。
 */
export const useAppStore = defineStore("app", () => {
  const currentView = ref<ViewKey>("bank");

  /** 当前刷题会话进度（持久化在 Rust 层 practice_session 表） */
  const session = ref<PracticeSession | null>(null);

  function setView(view: ViewKey) {
    currentView.value = view;
  }

  return { currentView, session, setView };
});
