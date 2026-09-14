// 前端 AI 直调服务：使用 axios + DeepSeek Chat Completions API。
// 判题走 /v1/chat/completions（JSON 模式 + 关闭思考），错题解析走纯文本。
// API Key 仅存内存（经 get_ai_config_full 从 Rust 侧 SQLite 读入），不落 localStorage / sessionStorage。
//
// 实测结论（2026-09）：
// - deepseek-flash 当前环境挂起（>120s 无返回），不可用
// - deepseek-v4-pro 正常（判题 ~1.5s）
// - thinking: { type: "disabled" } 可关闭思考，显著降低耗时与 token 消耗
// - response_format: { type: "json_object" } 可稳定输出 JSON

import axios, { AxiosError } from "axios";
import { getAiAnalysis, getAiConfigFull, saveAiAnalysis } from "../api";

interface FullConfig {
  base_url: string;
  api_key: string;
  model: string;
  judge_enabled: boolean;
  analysis_enabled: boolean;
}

/** 缓存的 AI 配置（仅内存；设置保存后调用 resetAiClient 清除） */
let cached: { base_url: string; api_key: string; model: string } | null = null;

/** 读取（或复用）AI 配置；未配置时抛错 */
async function ensureConfig() {
  if (cached) return cached;
  const cfg: FullConfig = await getAiConfigFull();
  if (!cfg.api_key) {
    throw new Error("未配置 API Key，请先到设置页配置");
  }
  if (!cfg.base_url) {
    throw new Error("未配置 Base URL");
  }
  cached = {
    base_url: cfg.base_url.trim().replace(/\/+$/, ""),
    api_key: cfg.api_key,
    model: cfg.model?.trim() || "deepseek-v4-pro",
  };
  return cached;
}

/** 重置缓存（设置保存后调用，使新配置立即生效） */
export function resetAiClient() {
  cached = null;
}

/** 把底层错误转成友好提示（带底层细节，便于排查） */
function describeError(err: unknown): string {
  if (axios.isAxiosError(err)) {
    const status = err.response?.status;
    const serverMsg = (err.response?.data as { error?: { message?: string } } | undefined)?.error?.message;
    if (status === 401) return serverMsg ? `API Key 无效（${serverMsg}）` : "API Key 无效，请到设置检查";
    if (status === 402) return "AI 账户余额不足，请充值";
    if (status === 422) return "请求参数错误（请检查模型名与 Base URL）";
    if (status === 429) return "请求过于频繁，请稍后再试";
    if (status === 500 || status === 503) return "AI 服务端故障/繁忙，请稍后重试";
    if (status) return serverMsg ? `AI 接口错误（HTTP ${status}）：${serverMsg}` : `AI 接口返回错误（HTTP ${status}）`;
    if (err.code === AxiosError.ECONNABORTED) {
      const t = (err.config as { timeout?: number } | undefined)?.timeout;
      return `AI 服务连接超时（${t ?? "?"}ms），可稍后重试`;
    }
    if (err.code === AxiosError.ERR_NETWORK) return `AI 服务连接失败：${err.message}`;
    return err.message;
  }
  return err instanceof Error ? err.message : String(err);
}

/** 发起一次 Chat Completions 请求，返回助手回复文本 */
async function chatCompletion(params: {
  messages: { role: "system" | "user"; content: string }[];
  temperature?: number;
  /** 是否启用 JSON 输出（response_format: json_object） */
  json?: boolean;
  maxTokens?: number;
  timeout?: number;
}): Promise<string> {
  const cfg = await ensureConfig();

  const body: Record<string, unknown> = {
    model: cfg.model,
    temperature: params.temperature ?? 1,
    thinking: { type: "disabled" },
    stream: false,
    messages: params.messages,
  };
  if (params.json) body.response_format = { type: "json_object" };
  if (params.maxTokens) body.max_tokens = params.maxTokens;

  const resp = await axios.request({
    baseURL: cfg.base_url,
    url: "/v1/chat/completions",
    method: "post",
    timeout: params.timeout ?? 15000,
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${cfg.api_key}`,
    },
    data: body,
  });

  const content = (resp.data as { choices?: { message?: { content?: unknown } }[] } | undefined)
    ?.choices?.[0]?.message?.content;
  return typeof content === "string" ? content : "";
}

/** 测试连接：最小请求验证连通性与 Key */
export async function testAiConnectionJs(): Promise<string> {
  try {
    const text = await chatCompletion({
      messages: [
        { role: "system", content: "你是连接测试助手，请简短回复。" },
        { role: "user", content: "回复：ok" },
      ],
      temperature: 0,
      maxTokens: 64,
      timeout: 15000,
    });
    if (!text.trim()) {
      throw new Error("连接成功但模型响应为空");
    }
    return `连接成功，模型响应：${text.trim()}`;
  } catch (e) {
    throw new Error(describeError(e));
  }
}

/** AI 判题（简答题）：JSON 输出，返回判分与理由 */
export async function judgeEssayJs(params: {
  content: string;
  answer: string;
  userAnswer: string;
}): Promise<{ correct: number; reason: string }> {
  try {
    const text = await chatCompletion({
      json: true,
      temperature: 0.2,
      maxTokens: 300,
      timeout: 10000,
      messages: [
        {
          role: "system",
          content:
            "你是严格的中文阅卷老师。请判断学生作答是否正确，以 json 格式输出，不要输出其他任何内容。\n" +
            "期望输出的 JSON 样例：\n" +
            '{"correct": 1, "reason": "回答正确，要点齐全"}\n' +
            "其中 correct 取 0（错误）或 1（正确），reason 为不超过 80 字的简短理由。",
        },
        {
          role: "user",
          content: `题干：${params.content}\n参考答案：${params.answer}\n学生作答：${params.userAnswer}`,
        },
      ],
    });
    let parsed: { correct?: unknown; reason?: unknown };
    try {
      parsed = JSON.parse(text);
    } catch {
      throw new Error(`AI 判题返回非 JSON：${text.slice(0, 120)}`);
    }
    const correct = parsed.correct === 1 ? 1 : 0;
    return {
      correct,
      reason: typeof parsed.reason === "string" && parsed.reason ? parsed.reason : "（无理由）",
    };
  } catch (e) {
    throw new Error(describeError(e));
  }
}

/** 简单字符串哈希（djb2），用于 AI 解析缓存 key（题目ID + 作答哈希） */
function hashString(s: string): string {
  let h = 5381;
  for (let i = 0; i < s.length; i++) {
    h = ((h << 5) + h + s.charCodeAt(i)) >>> 0;
  }
  return h.toString(16);
}

/** AI 错题解析：答错后生成针对性解析（纯文本）。
 *  按「题目ID + 作答哈希」本地缓存，相同错误直接读缓存，不重复调用计费（业务文档 6.4）。 */
export async function analyzeMistakeJs(params: {
  questionId: number;
  content: string;
  answer: string;
  userAnswer: string;
  analysis?: string;
}): Promise<string> {
  try {
    const answerHash = hashString(params.userAnswer);
    // 命中缓存直接返回
    const cachedText = await getAiAnalysis(params.questionId, answerHash);
    if (cachedText) return cachedText;

    const ref = params.analysis ? `\n原题解析：${params.analysis}` : "";
    const text = await chatCompletion({
      temperature: 0.4,
      maxTokens: 400,
      timeout: 15000,
      messages: [
        {
          role: "system",
          content:
            "你是耐心的刷题助教。请用中文简要说明学生错在哪里、正确思路是什么。120 字以内，直接输出解析文本，不要编号。",
        },
        {
          role: "user",
          content: `题干：${params.content}\n参考答案：${params.answer}\n学生作答：${params.userAnswer}${ref}`,
        },
      ],
    });

    // 写回缓存（静默失败，不影响展示）
    if (text) {
      saveAiAnalysis(params.questionId, answerHash, text).catch(() => {});
    }
    return text;
  } catch (e) {
    throw new Error(describeError(e));
  }
}
