import { defineStore } from "pinia";
import { ref } from "vue";
import type { PracticeSession } from "../types";

/** 主界面视图 */
export type ViewKey = "bank" | "practice" | "fault" | "collect" | "stats" | "backup";

/** 刷题来源：从题库集、错题本或收藏发起刷题 */
export interface PracticeSource {
  kind: "bank" | "fault" | "collect";
  bankId?: number | null;
  tagId?: number | null;
}

/**
 * 全局应用状态（技术文档 3：Pinia 维护内存状态）。
 */
export const useAppStore = defineStore("app", () => {
  const currentView = ref<ViewKey>("bank");

  /** 当前刷题会话进度（持久化在 Rust 层 practice_session 表） */
  const session = ref<PracticeSession | null>(null);

  /** 跨视图跳转：错题本/收藏页发起刷题时设置来源后切换到刷题视图 */
  const practiceSource = ref<PracticeSource | null>(null);

  function setView(view: ViewKey) {
    currentView.value = view;
  }

  /** 从指定来源发起刷题并跳转到刷题页 */
  function startPractice(source: PracticeSource) {
    practiceSource.value = source;
    currentView.value = "practice";
  }

  return { currentView, session, practiceSource, setView, startPractice };
});
