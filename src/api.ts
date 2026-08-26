import { invoke } from "@tauri-apps/api/core";
import type { Question, QuestionBank, StatsSummary } from "./types";

/**
 * Rust Command 统一封装。
 * 架构约束（技术文档 3）：前端不直接操作数据库与文件，所有持久化操作走 Tauri Command。
 */

export function getBanks(): Promise<QuestionBank[]> {
  return invoke<QuestionBank[]>("get_banks");
}

export function createBank(name: string): Promise<QuestionBank> {
  return invoke<QuestionBank>("create_bank", { name });
}

export function deleteBank(bankId: number): Promise<void> {
  return invoke<void>("delete_bank", { bankId });
}

export function getQuestions(bankId: number | null): Promise<Question[]> {
  return invoke<Question[]>("get_questions", { bankId });
}

export function getStats(): Promise<StatsSummary> {
  return invoke<StatsSummary>("get_stats");
}
