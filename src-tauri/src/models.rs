//! 数据模型：与 SQLite 表结构、Command 入参/返回值、备份包结构一一对应。

use serde::{Deserialize, Serialize};

/// 合法题型枚举（业务文档 4.1）
pub const VALID_Q_TYPES: [&str; 5] = ["single", "multi", "judge", "essay", "fill"];

/// 题库集（question_bank），question_count 为聚合出的题目数
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bank {
    pub id: i64,
    pub name: String,
    pub create_time: i64,
    pub question_count: i64,
}

/// 题目行（question 表原始字段，备份/查询共用）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuestionRow {
    pub id: i64,
    pub bank_id: i64,
    /// single / multi / judge / essay / fill
    pub q_type: String,
    /// 题干
    pub content: String,
    /// JSON 字符串存储选项，简答/填空为 null
    pub options: Option<String>,
    /// 答案；填空多答案英文分号 ; 分隔
    pub answer: String,
    /// 解析
    pub analysis: Option<String>,
    pub create_time: i64,
    pub update_time: i64,
}

/// 题目（含标签名列表与收藏状态，返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: i64,
    pub bank_id: i64,
    pub q_type: String,
    pub content: String,
    pub options: Option<String>,
    pub answer: String,
    pub analysis: Option<String>,
    pub create_time: i64,
    pub update_time: i64,
    pub tags: Vec<String>,
    /// 是否已收藏 0/1（聚合自 answer_record）
    pub is_collect: i64,
}

impl From<QuestionRow> for Question {
    fn from(row: QuestionRow) -> Self {
        Self {
            id: row.id,
            bank_id: row.bank_id,
            q_type: row.q_type,
            content: row.content,
            options: row.options,
            answer: row.answer,
            analysis: row.analysis,
            create_time: row.create_time,
            update_time: row.update_time,
            tags: Vec::new(),
            is_collect: 0,
        }
    }
}

/// 新建/更新题目入参
#[derive(Debug, Clone, Deserialize)]
pub struct QuestionInput {
    pub bank_id: i64,
    pub q_type: String,
    pub content: String,
    pub options: Option<String>,
    pub answer: String,
    pub analysis: Option<String>,
    pub tags: Vec<String>,
}

/// CSV 批量导入的题目行（bank_name 为 CSV「所属题库集」列，可选；
/// 为空时归入命令参数指定的默认题库集）
#[derive(Debug, Clone, Deserialize)]
pub struct NewQuestion {
    pub q_type: String,
    pub content: String,
    pub options: Option<String>,
    pub answer: String,
    pub analysis: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub bank_name: Option<String>,
}

/// 批量导入结果
#[derive(Debug, Clone, Serialize)]
pub struct BatchInsertResult {
    pub inserted: i64,
    pub skipped: i64,
    /// 跳过原因（题干重复、字段非法等），供前端汇总提示
    pub skipped_details: Vec<String>,
}

/// 标签（tag）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

/// 题目-标签关联行（question_tag，备份用）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuestionTagRow {
    pub question_id: i64,
    pub tag_id: i64,
}

/// 题库集原表行（question_bank，备份用）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuestionBankRow {
    pub id: i64,
    pub name: String,
    pub create_time: i64,
}

/// 做题记录（answer_record）：同时承载错题、收藏、作答记录
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AnswerRecord {
    pub id: i64,
    pub question_id: i64,
    /// 用户作答文本
    pub user_answer: Option<String>,
    /// 机器判分 0错误 1正确；简答为 null
    pub machine_result: Option<i64>,
    /// 人工覆写结果 null/0/1，优先级高于 machine_result
    pub manual_result: Option<i64>,
    /// 是否错题 0否 1是
    pub is_fault: i64,
    /// 是否收藏 0否 1是
    pub is_collect: i64,
    /// 做错累计次数
    pub fault_count: i64,
    pub finish_time: i64,
}

/// 提交作答入参
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitAnswerInput {
    pub question_id: i64,
    pub user_answer: Option<String>,
    /// single/multi/judge/fill 由前端判分后传入；essay 为 null
    pub machine_result: Option<i64>,
    /// essay 手动标记；fill 覆写时传入；优先级最高
    pub manual_result: Option<i64>,
}

/// 刷题会话（practice_session）：保存中途退出的刷题进度
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PracticeSession {
    pub id: i64,
    pub bank_id: Option<i64>,
    /// JSON 存储筛选标签
    pub tag_filter: Option<String>,
    pub current_index: i64,
    /// order / random
    pub practice_mode: String,
    pub create_time: i64,
}

/// 全局统计汇总（技术文档 5.4）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StatsSummary {
    pub total_questions: i64,
    pub total_records: i64,
    pub correct_count: i64,
    pub wrong_count: i64,
    /// 整体正确率，百分数（0-100），保留两位小数
    pub correct_rate: f64,
}

/// 分题库统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStats {
    pub bank_id: i64,
    pub bank_name: String,
    pub total_questions: i64,
    pub total_records: i64,
    pub correct_count: i64,
    pub wrong_count: i64,
    pub correct_rate: f64,
}

/// 备份包完整结构（技术文档 5.5 / 6.2：.qpbackup 为 JSON 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub version: u32,
    /// 导出时间戳（毫秒）
    pub exported_at: i64,
    pub question_banks: Vec<QuestionBankRow>,
    pub questions: Vec<QuestionRow>,
    pub tags: Vec<Tag>,
    pub question_tags: Vec<QuestionTagRow>,
    pub answer_records: Vec<AnswerRecord>,
    pub practice_sessions: Vec<PracticeSession>,
}

/// 备份导出/恢复结果
#[derive(Debug, Clone, Serialize)]
pub struct BackupResult {
    pub path: String,
    pub question_banks: usize,
    pub questions: usize,
    pub tags: usize,
    pub question_tags: usize,
    pub answer_records: usize,
    pub practice_sessions: usize,
}
