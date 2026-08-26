import type { QuestionType } from "../types";

/**
 * 填空题机器判分逻辑（技术文档 7.1）
 * 多标准答案使用英文分号 `;` 分隔，命中任意一个即正确。
 * 规则：去除首尾空格，大小写敏感，中间空格原样比对。
 *
 * @param userInput 用户输入
 * @param stdAnswer 原始标准答案，多答案用 ; 分割
 * @returns boolean 是否机器判定正确
 */
export function judgeFillAnswer(userInput: string, stdAnswer: string): boolean {
  const input = userInput.trim();
  const stdList = stdAnswer.split(";").map((s) => s.trim());
  return stdList.includes(input);
}

/**
 * 多选题判分（技术文档 7.2）
 * 标准答案逗号分割，用户选项集合必须完全一致，多选、少选、错选判定错误。
 */
export function judgeMultiAnswer(userSelected: string[], stdAnswer: string): boolean {
  const stdList = stdAnswer
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean)
    .sort();
  const selected = [...userSelected]
    .map((s) => s.trim())
    .filter(Boolean)
    .sort();
  if (stdList.length !== selected.length) return false;
  return stdList.every((v, i) => v === selected[i]);
}

/** 单选题判分：选项文本去除首尾空格后完全一致 */
export function judgeSingleAnswer(userSelected: string, stdAnswer: string): boolean {
  return userSelected.trim() === stdAnswer.trim();
}

/** 判断题判分：true/false，忽略大小写与首尾空格 */
export function judgeJudgeAnswer(userSelected: string, stdAnswer: string): boolean {
  return userSelected.trim().toLowerCase() === stdAnswer.trim().toLowerCase();
}

/**
 * 按题型分派机器判分。
 * essay 简答题无机器判分，返回 null（手动标记对错）。
 */
export function machineJudge(
  qType: QuestionType,
  userAnswer: string | string[],
  stdAnswer: string,
): boolean | null {
  switch (qType) {
    case "single":
      return judgeSingleAnswer(String(userAnswer), stdAnswer);
    case "multi":
      return judgeMultiAnswer(
        Array.isArray(userAnswer) ? userAnswer : [String(userAnswer)],
        stdAnswer,
      );
    case "judge":
      return judgeJudgeAnswer(String(userAnswer), stdAnswer);
    case "fill":
      return judgeFillAnswer(String(userAnswer), stdAnswer);
    case "essay":
    default:
      return null;
  }
}
