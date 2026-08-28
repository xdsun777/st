import Papa from "papaparse";
import type { NewQuestion, QuestionType } from "../types";

/**
 * CSV 导入解析与字段校验（技术文档 5.1 / 6.1）。
 *
 * 字段规范（表头，业务文档 4.2）：
 * 题目ID(否) | 题目类型(是) | 题干(是) | 选项(否) | 正确答案(是) | 解析(否) | 标签(否) | 所属题库集(否)
 *
 * 校验规则：
 * 1. 仅支持 .csv（文件选择处拦截）
 * 2. 题干为空、类型非法、答案格式错误 → 跳过该行，输出错误行号提示
 * 3. 重复题干由 Rust 层 batch_insert_questions 处理（保留原题）
 */

export interface CsvParseError {
  /** 数据行号（含表头，与用户所见一致） */
  row: number;
  message: string;
}

export interface CsvParseResult {
  rows: NewQuestion[];
  errors: CsvParseError[];
}

const VALID_TYPES: QuestionType[] = ["single", "multi", "judge", "essay", "fill"];

/** 表头候选（支持中英文与空格差异，宽松匹配） */
const HEADER_ALIASES: Record<string, string[]> = {
  type: ["题目类型", "类型", "type"],
  content: ["题干", "题目", "content"],
  options: ["选项", "options"],
  answer: ["正确答案", "答案", "answer"],
  analysis: ["解析", "analysis"],
  tags: ["标签", "tags"],
  bank: ["所属题库集", "题库集", "bank"],
};

function pickField(row: Record<string, string>, aliases: string[]): string | undefined {
  for (const key of Object.keys(row)) {
    const normalized = key.trim();
    if (aliases.some((a) => a.toLowerCase() === normalized.toLowerCase())) {
      return row[key];
    }
  }
  return undefined;
}

/** 校验并规范化题型 */
function parseType(raw: string | undefined): QuestionType | null {
  const value = (raw ?? "").trim().toLowerCase();
  return (VALID_TYPES as string[]).includes(value) ? (value as QuestionType) : null;
}

/** 解析标签：英文逗号分隔（业务文档 4.2：多个标签英文逗号分隔） */
function parseTags(raw: string | undefined): string[] {
  if (!raw || !raw.trim()) return [];
  return raw
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
}

/** 解析选项：支持逗号、换行、| 、；分隔；存为 JSON 字符串数组 */
function parseOptions(raw: string | undefined, qType: QuestionType): string | null {
  if (!raw || !raw.trim()) return null;
  // 先按逗号拆分（CSV 引号内的逗号分隔选项），拆分不足 2 个时再按换行/|/；拆分
  let parts = raw
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  if (parts.length < 2) {
    parts = raw
      .split(/\r?\n|\||；/)
      .map((s) => s.trim())
      .filter(Boolean);
  }
  if (parts.length === 0) return null;
  if (qType === "single" || qType === "multi") {
    return JSON.stringify(parts);
  }
  return null;
}

/** 答案规范化：多选答案兼容分号分隔（data.csv 使用 A;B;C），统一为逗号分隔 */
function normalizeAnswer(raw: string, qType: QuestionType): string {
  let answer = raw.trim();
  if (qType === "multi") {
    answer = answer.replace(/[;；]/g, ",");
  }
  return answer;
}

/**
 * 单选/多选答案若是选项字母（A/B/C…），映射为对应选项文本。
 * 例：选项 ["MySQL","Redis"]，答案 "A" → "MySQL"；多选 "A,B" → "MySQL,Redis"。
 * 若答案本就是选项文本（非单字母），原样返回。
 */
function mapLetterAnswer(answer: string, optionsJson: string | null): string {
  if (!optionsJson) return answer;
  let options: string[];
  try {
    options = JSON.parse(optionsJson);
    if (!Array.isArray(options)) return answer;
  } catch {
    return answer;
  }

  const tokens = answer
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  const mapped = tokens.map((token) => {
    if (/^[A-Z]$/.test(token)) {
      const index = token.charCodeAt(0) - "A".charCodeAt(0);
      return options[index] ?? token;
    }
    return token;
  });
  return mapped.join(",");
}

/** 答案格式校验（宽松：非空即可；判断题严格要求 true/false） */
function validateAnswer(raw: string | undefined, qType: QuestionType): string | null {
  const answer = (raw ?? "").trim();
  if (!answer) return "答案为空";
  if (qType === "judge" && !["true", "false"].includes(answer.toLowerCase())) {
    return "判断题答案必须为 true/false";
  }
  return null;
}

/**
 * 解析 CSV 文本。返回合法题目行与错误行列表。
 * @param text CSV 原始文本
 */
export function parseCsv(text: string): CsvParseResult {
  const result = Papa.parse<Record<string, string>>(text, {
    header: true,
    skipEmptyLines: "greedy",
  });

  const rows: NewQuestion[] = [];
  const errors: CsvParseError[] = [];

  result.data.forEach((record, index) => {
    // 数据行号 = 表头 1 行 + 数据行偏移（PapaParse 不返回表头）
    const rowNo = index + 2;

    const typeRaw = pickField(record, HEADER_ALIASES.type);
    const contentRaw = pickField(record, HEADER_ALIASES.content);
    const optionsRaw = pickField(record, HEADER_ALIASES.options);
    const answerRaw = pickField(record, HEADER_ALIASES.answer);
    const analysisRaw = pickField(record, HEADER_ALIASES.analysis);
    const tagsRaw = pickField(record, HEADER_ALIASES.tags);
    const bankRaw = pickField(record, HEADER_ALIASES.bank);

    // 跳过完全空行（skipEmptyLines 已处理，双保险）
    if (!contentRaw?.trim() && !typeRaw?.trim() && !answerRaw?.trim()) return;

    const qType = parseType(typeRaw);
    if (!qType) {
      errors.push({ row: rowNo, message: `题型非法（${(typeRaw ?? "").trim() || "空"}），应为 single/multi/judge/essay/fill` });
      return;
    }

    const content = (contentRaw ?? "").trim();
    if (!content) {
      errors.push({ row: rowNo, message: "题干为空" });
      return;
    }

    const answerError = validateAnswer(answerRaw, qType);
    if (answerError) {
      errors.push({ row: rowNo, message: answerError });
      return;
    }

    const options = parseOptions(optionsRaw, qType);
    const answer = mapLetterAnswer(normalizeAnswer(answerRaw ?? "", qType), options);

    rows.push({
      q_type: qType,
      content,
      options,
      answer,
      analysis: analysisRaw?.trim() ? analysisRaw.trim() : null,
      tags: parseTags(tagsRaw),
      bank_name: bankRaw?.trim() ? bankRaw.trim() : null,
    });
  });

  return { rows, errors };
}
