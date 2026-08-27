//! 公共工具：连接池获取、时间戳、判分结果解析、题型/标签校验、题目行解析。

use sqlx::Row;
use tauri_plugin_sql::DbPool;

use crate::models::{Question, QuestionRow};

/// 当前毫秒时间戳
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// 从插件管理的 DbPool 中取出 SQLite 连接池。
/// 本项目仅启用 sqlite 特性（技术文档 2：本地数据库 SQLite），
/// DbPool 此时只有 Sqlite 变体，无需处理其他数据库分支。
pub fn sqlite_pool(pool: &DbPool) -> Result<&sqlx::SqlitePool, String> {
    match pool {
        DbPool::Sqlite(pool) => Ok(pool),
    }
}

/// 解析最终判分结果：manual_result 优先级高于 machine_result（业务文档 7.3）。
pub fn resolve_result(machine_result: Option<i64>, manual_result: Option<i64>) -> Option<i64> {
    manual_result.or(machine_result)
}

/// 校验题型是否合法（业务文档 4.1）
pub fn is_valid_q_type(q_type: &str) -> bool {
    crate::models::VALID_Q_TYPES.contains(&q_type)
}

/// 校验标签名：非空、不含逗号（group_concat 与 CSV 标签均以逗号分隔）
pub fn is_valid_tag_name(name: &str) -> bool {
    let name = name.trim();
    !name.is_empty() && !name.contains(',') && !name.contains(';')
}

/// group_concat 结果按逗号拆分（标签名本身不允许含逗号）
pub fn split_tags(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        Vec::new()
    } else {
        raw.split(',').map(|s| s.trim().to_string()).collect()
    }
}

/// 从带 tags 聚合列（GROUP_CONCAT）的查询行构造题目。
/// 查询列别名要求：id, bank_id, q_type, content, options, answer, analysis,
/// create_time, update_time, tags。
pub fn row_to_question(row: &sqlx::sqlite::SqliteRow) -> Result<Question, String> {
    let tags: String = row.try_get("tags").map_err(|e| e.to_string())?;
    let mut question = Question::from(QuestionRow {
        id: row.try_get("id").map_err(|e| e.to_string())?,
        bank_id: row.try_get("bank_id").map_err(|e| e.to_string())?,
        q_type: row.try_get("q_type").map_err(|e| e.to_string())?,
        content: row.try_get("content").map_err(|e| e.to_string())?,
        options: row.try_get("options").map_err(|e| e.to_string())?,
        answer: row.try_get("answer").map_err(|e| e.to_string())?,
        analysis: row.try_get("analysis").map_err(|e| e.to_string())?,
        create_time: row.try_get("create_time").map_err(|e| e.to_string())?,
        update_time: row.try_get("update_time").map_err(|e| e.to_string())?,
    });
    question.tags = split_tags(&tags);
    Ok(question)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_result_manual_priority() {
        // manual 优先
        assert_eq!(resolve_result(Some(0), Some(1)), Some(1));
        assert_eq!(resolve_result(Some(1), Some(0)), Some(0));
        // 无 manual 时用 machine
        assert_eq!(resolve_result(Some(1), None), Some(1));
        assert_eq!(resolve_result(None, None), None);
        // essay：无机器判分，仅 manual
        assert_eq!(resolve_result(None, Some(1)), Some(1));
    }

    #[test]
    fn test_is_valid_q_type() {
        assert!(is_valid_q_type("single"));
        assert!(is_valid_q_type("multi"));
        assert!(is_valid_q_type("judge"));
        assert!(is_valid_q_type("essay"));
        assert!(is_valid_q_type("fill"));
        assert!(!is_valid_q_type(""));
        assert!(!is_valid_q_type("choice"));
    }

    #[test]
    fn test_is_valid_tag_name() {
        assert!(is_valid_tag_name("数据库"));
        assert!(is_valid_tag_name(" 第三章 "));
        assert!(!is_valid_tag_name(""));
        assert!(!is_valid_tag_name("  "));
        assert!(!is_valid_tag_name("a,b"));
        assert!(!is_valid_tag_name("a;b"));
    }
}
