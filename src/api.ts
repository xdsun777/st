import { invoke } from "@tauri-apps/api/core";
import type {
  AnswerRecord,
  BackupResult,
  BankStats,
  BatchInsertResult,
  NewQuestion,
  PracticeSession,
  Question,
  QuestionBank,
  QuestionInput,
  StatsSummary,
  SubmitAnswerInput,
  Tag,
} from "./types";

/**
 * Rust Command 统一封装。
 * 架构约束（技术文档 3）：前端不直接操作数据库与文件，所有持久化操作走 Tauri Command。
 */

// ---------- 题库集 ----------
export function getBanks(): Promise<QuestionBank[]> {
  return invoke<QuestionBank[]>("get_banks");
}

export function createBank(name: string): Promise<QuestionBank> {
  return invoke<QuestionBank>("create_bank", { name });
}

export function renameBank(bankId: number, name: string): Promise<void> {
  return invoke<void>("rename_bank", { bankId, name });
}

export function deleteBank(bankId: number): Promise<void> {
  return invoke<void>("delete_bank", { bankId });
}

// ---------- 题目 ----------
export function getQuestions(bankId: number | null, tagId: number | null = null): Promise<Question[]> {
  return invoke<Question[]>("get_questions", { bankId, tagId });
}

export function getQuestion(questionId: number): Promise<Question> {
  return invoke<Question>("get_question", { questionId });
}

export function createQuestion(input: QuestionInput): Promise<Question> {
  return invoke<Question>("create_question", { input });
}

export function updateQuestion(questionId: number, input: QuestionInput): Promise<Question> {
  return invoke<Question>("update_question", { questionId, input });
}

export function deleteQuestion(questionId: number): Promise<void> {
  return invoke<void>("delete_question", { questionId });
}

export function batchInsertQuestions(bankId: number, questions: NewQuestion[]): Promise<BatchInsertResult> {
  return invoke<BatchInsertResult>("batch_insert_questions", { bankId, questions });
}

// ---------- 标签 ----------
export function listTags(): Promise<Tag[]> {
  return invoke<Tag[]>("list_tags");
}

export function createTag(name: string): Promise<Tag> {
  return invoke<Tag>("create_tag", { name });
}

export function deleteTag(tagId: number): Promise<void> {
  return invoke<void>("delete_tag", { tagId });
}

// ---------- 刷题会话 ----------
export function savePracticeSession(
  bankId: number | null,
  tagFilter: string | null,
  currentIndex: number,
  practiceMode: string,
): Promise<PracticeSession> {
  return invoke<PracticeSession>("save_practice_session", {
    bankId,
    tagFilter,
    currentIndex,
    practiceMode,
  });
}

export function loadPracticeSession(): Promise<PracticeSession | null> {
  return invoke<PracticeSession | null>("load_practice_session");
}

export function clearPracticeSession(): Promise<void> {
  return invoke<void>("clear_practice_session");
}

// ---------- 做题记录 / 错题 / 收藏 ----------
export function submitAnswer(input: SubmitAnswerInput): Promise<AnswerRecord> {
  return invoke<AnswerRecord>("submit_answer", { input });
}

export function updateFault(questionId: number, isFault: boolean): Promise<void> {
  return invoke<void>("update_fault", { questionId, isFault });
}

export function updateCollect(questionId: number, isCollect: boolean): Promise<void> {
  return invoke<void>("update_collect", { questionId, isCollect });
}

export function getFaultQuestions(tagId: number | null = null): Promise<Question[]> {
  return invoke<Question[]>("get_fault_questions", { tagId });
}

export function getCollectQuestions(tagId: number | null = null): Promise<Question[]> {
  return invoke<Question[]>("get_collect_questions", { tagId });
}

export function getAnswerRecords(questionId: number): Promise<AnswerRecord[]> {
  return invoke<AnswerRecord[]>("get_answer_records", { questionId });
}

// ---------- 统计 ----------
export function getStats(): Promise<StatsSummary> {
  return invoke<StatsSummary>("get_stats");
}

export function getBankStats(): Promise<BankStats[]> {
  return invoke<BankStats[]>("get_bank_stats");
}

// ---------- 备份 ----------
export function exportBackup(path: string): Promise<BackupResult> {
  return invoke<BackupResult>("export_backup", { path });
}

export function importBackup(path: string): Promise<BackupResult> {
  return invoke<BackupResult>("import_backup", { path });
}
