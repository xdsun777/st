/** 题目类型枚举（业务文档 4.1） */
export type QuestionType = "single" | "multi" | "judge" | "essay" | "fill";

/** 刷题模式：order 顺序 / random 随机 */
export type PracticeMode = "order" | "random";

/** 题库集 question_bank */
export interface QuestionBank {
  id: number;
  name: string;
  create_time: number;
  question_count: number;
}

/** 题目 question */
export interface Question {
  id: number;
  bank_id: number;
  q_type: QuestionType;
  /** 题干 */
  content: string;
  /** JSON 字符串存储选项数组，简答/填空为 null */
  options: string | null;
  /** 答案；填空多答案英文分号 ; 分隔 */
  answer: string;
  /** 解析 */
  analysis: string | null;
  create_time: number;
  update_time: number;
}

/** 标签 tag */
export interface Tag {
  id: number;
  name: string;
}

/** 做题记录 answer_record：同时承载错题、收藏、作答记录 */
export interface AnswerRecord {
  id: number;
  question_id: number;
  /** 用户作答文本 */
  user_answer: string | null;
  /** 机器判分 0错误 1正确；简答为 null */
  machine_result: number | null;
  /** 人工覆写结果 null/0/1，优先级高于 machine_result */
  manual_result: number | null;
  /** 是否错题 0否 1是 */
  is_fault: number;
  /** 是否收藏 0否 1是 */
  is_collect: number;
  /** 做错累计次数 */
  fault_count: number;
  finish_time: number;
}

/** 刷题会话 practice_session：保存中途退出的刷题进度 */
export interface PracticeSession {
  id: number;
  bank_id: number | null;
  /** JSON 存储筛选标签 */
  tag_filter: string | null;
  current_index: number;
  practice_mode: PracticeMode;
  create_time: number;
}

/** 统计汇总（技术文档 5.4） */
export interface StatsSummary {
  total_questions: number;
  total_records: number;
  correct_count: number;
  wrong_count: number;
  /** 整体正确率，百分数（0-100），保留两位小数 */
  correct_rate: number;
}
