//! AI 判分结果写回（技术文档 6.5）。
//! Rust 端不发起任何网络请求；AI 调用由前端 JS（axios）完成，
//! 本模块仅负责把前端判分结果持久化到 answer_record，同步统计与错题本。

use tauri::State;

use crate::common::{now_ms, sqlite_pool};

/// 前端 JS 判题后将 AI 结果写回数据库（统计与错题本同步）。
/// 写库逻辑：更新 ai_result，并按最终结果同步 is_fault 与 fault_count。
#[tauri::command]
pub async fn update_ai_result(
    pool: State<'_, sqlx::SqlitePool>,
    record_id: i64,
    ai_result: i64,
) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let row: (i64,) = sqlx::query_as("SELECT question_id FROM answer_record WHERE id = ?1")
        .bind(record_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("做题记录不存在")?;
    let question_id = row.0;
    let correct = if ai_result == 1 { 1 } else { 0 };
    let hist: (i64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(fault_count), 0) FROM answer_record WHERE question_id = ?1",
    )
    .bind(question_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let is_fault = if correct == 0 { 1 } else { 0 };
    let fault_count = if correct == 0 { hist.0 + 1 } else { hist.0 };
    sqlx::query("UPDATE answer_record SET ai_result = ?1, is_fault = ?2, fault_count = ?3 WHERE id = ?4")
        .bind(correct)
        .bind(is_fault)
        .bind(fault_count)
        .bind(record_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("UPDATE answer_record SET is_fault = ?1 WHERE question_id = ?2")
        .bind(is_fault)
        .bind(question_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 读取 AI 错题解析缓存（按「题目ID + 作答哈希」命中，业务文档 6.4）。
#[tauri::command]
pub async fn get_ai_analysis(
    pool: State<'_, sqlx::SqlitePool>,
    question_id: i64,
    answer_hash: String,
) -> Result<Option<String>, String> {
    let pool = sqlite_pool(&pool)?;
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT analysis FROM ai_analysis_cache WHERE question_id = ?1 AND answer_hash = ?2",
    )
    .bind(question_id)
    .bind(&answer_hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map(|r| r.0))
}

/// 写入 AI 错题解析缓存（存在则覆盖，避免相同错误重复计费）。
#[tauri::command]
pub async fn save_ai_analysis(
    pool: State<'_, sqlx::SqlitePool>,
    question_id: i64,
    answer_hash: String,
    analysis: String,
) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;
    sqlx::query(
        "INSERT INTO ai_analysis_cache (question_id, answer_hash, analysis, create_time)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(question_id, answer_hash)
         DO UPDATE SET analysis = excluded.analysis, create_time = excluded.create_time",
    )
    .bind(question_id)
    .bind(&answer_hash)
    .bind(&analysis)
    .bind(now_ms())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
