//! Tauri Command 层：所有持久化操作统一在 Rust 侧完成，前端只传参数。
//!
//! 初始化阶段实现题库集增删查、题目查询、统计聚合，用于验证
//! 「前端 UI → Tauri Command → Rust(SQLite) → 本地磁盘」数据链路。

use tauri::State;
use tauri_plugin_sql::DbPool;

use crate::models::{Bank, Question, StatsSummary};

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// 从插件管理的 DbPool 中取出 SQLite 连接池。
/// 本项目仅启用 sqlite 特性（技术文档 2：本地数据库 SQLite），
/// DbPool 此时只有 Sqlite 变体，无需处理其他数据库分支。
fn sqlite_pool(pool: &DbPool) -> Result<&sqlx::SqlitePool, String> {
    match pool {
        DbPool::Sqlite(pool) => Ok(pool),
    }
}

/// 题库集列表（含各题库集题目数，LEFT JOIN 聚合）
#[tauri::command]
pub async fn get_banks(pool: State<'_, DbPool>) -> Result<Vec<Bank>, String> {
    let pool = sqlite_pool(&pool)?;
    let banks = sqlx::query_as::<_, Bank>(
        "SELECT b.id, b.name, b.create_time, COUNT(q.id) AS question_count
         FROM question_bank b
         LEFT JOIN question q ON q.bank_id = b.id
         GROUP BY b.id
         ORDER BY b.id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(banks)
}

/// 新建题库集
#[tauri::command]
pub async fn create_bank(pool: State<'_, DbPool>, name: String) -> Result<Bank, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("题库集名称不能为空".into());
    }
    let pool = sqlite_pool(&pool)?;
    let now = now_ms();
    let id = sqlx::query("INSERT INTO question_bank (name, create_time) VALUES (?1, ?2)")
        .bind(name)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();
    Ok(Bank {
        id,
        name: name.to_string(),
        create_time: now,
        question_count: 0,
    })
}

/// 删除题库集：显式按层级删除题目、做题记录、标签关联、刷题会话，
/// 不依赖外键开关，确保「删除题库集时集合内所有题目一并删除」。
#[tauri::command]
pub async fn delete_bank(pool: State<'_, DbPool>, bank_id: i64) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;

    sqlx::query(
        "DELETE FROM answer_record WHERE question_id IN (SELECT id FROM question WHERE bank_id = ?1)",
    )
    .bind(bank_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "DELETE FROM question_tag WHERE question_id IN (SELECT id FROM question WHERE bank_id = ?1)",
    )
    .bind(bank_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM question WHERE bank_id = ?1")
        .bind(bank_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM practice_session WHERE bank_id = ?1")
        .bind(bank_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM question_bank WHERE id = ?1")
        .bind(bank_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 查询题目：bank_id 为 null 时返回全部题目
#[tauri::command]
pub async fn get_questions(
    pool: State<'_, DbPool>,
    bank_id: Option<i64>,
) -> Result<Vec<Question>, String> {
    let pool = sqlite_pool(&pool)?;
    let questions = sqlx::query_as::<_, Question>(
        "SELECT id, bank_id, q_type, content, options, answer, analysis, create_time, update_time
         FROM question
         WHERE (?1 IS NULL OR bank_id = ?1)
         ORDER BY id",
    )
    .bind(bank_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(questions)
}

/// 全局统计：总题量、总刷题量、正确数、错误数、整体正确率。
/// manual_result 优先于 machine_result（业务文档 7.3）。
#[tauri::command]
pub async fn get_stats(pool: State<'_, DbPool>) -> Result<StatsSummary, String> {
    let pool = sqlite_pool(&pool)?;
    let row: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT
           (SELECT COUNT(*) FROM question) AS total_questions,
           (SELECT COUNT(*) FROM answer_record) AS total_records,
           (SELECT COUNT(*) FROM answer_record
              WHERE COALESCE(manual_result, machine_result) = 1) AS correct_count,
           (SELECT COUNT(*) FROM answer_record
              WHERE COALESCE(manual_result, machine_result) = 0) AS wrong_count",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let (total_questions, total_records, correct_count, wrong_count) = row;
    let judged = correct_count + wrong_count;
    let correct_rate = if judged > 0 {
        ((correct_count as f64 / judged as f64) * 10000.0).round() / 100.0
    } else {
        0.0
    };

    Ok(StatsSummary {
        total_questions,
        total_records,
        correct_count,
        wrong_count,
        correct_rate,
    })
}
