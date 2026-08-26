//! 与 SQLite 表结构一一对应的数据模型，用于 Command 返回值序列化与 sqlx 行映射。

use serde::{Deserialize, Serialize};

/// 题库集（question_bank），question_count 为聚合出的题目数
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bank {
    pub id: i64,
    pub name: String,
    pub create_time: i64,
    pub question_count: i64,
}

/// 题目（question）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Question {
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

/// 统计汇总（技术文档 5.4：Rust 层 SQL 聚合计算）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StatsSummary {
    pub total_questions: i64,
    pub total_records: i64,
    pub correct_count: i64,
    pub wrong_count: i64,
    /// 整体正确率，百分数（0-100），保留两位小数
    pub correct_rate: f64,
}
