//! 题库集（question_bank）CRUD 命令。
//!
//! 业务规则（业务文档 4.3）：
//! - 可新建、重命名、删除题库集
//! - 删除题库集：二次确认在前端完成；删除后集合内所有题目一并删除

use tauri::State;
use tauri_plugin_sql::DbPool;

use crate::common::{now_ms, sqlite_pool};
use crate::models::Bank;

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

/// 重命名题库集
#[tauri::command]
pub async fn rename_bank(
    pool: State<'_, DbPool>,
    bank_id: i64,
    name: String,
) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("题库集名称不能为空".into());
    }
    let pool = sqlite_pool(&pool)?;
    let result = sqlx::query("UPDATE question_bank SET name = ?1 WHERE id = ?2")
        .bind(name)
        .bind(bank_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    if result.rows_affected() == 0 {
        return Err("题库集不存在".into());
    }
    Ok(())
}

/// 删除题库集：显式按层级删除题目、做题记录、标签关联、刷题会话，
/// 不依赖外键开关，确保「删除题库集时集合内所有题目一并删除」。
#[tauri::command]
pub async fn delete_bank(pool: State<'_, DbPool>, bank_id: i64) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        "DELETE FROM answer_record WHERE question_id IN (SELECT id FROM question WHERE bank_id = ?1)",
    )
    .bind(bank_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "DELETE FROM question_tag WHERE question_id IN (SELECT id FROM question WHERE bank_id = ?1)",
    )
    .bind(bank_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM question WHERE bank_id = ?1")
        .bind(bank_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM practice_session WHERE bank_id = ?1")
        .bind(bank_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM question_bank WHERE id = ?1")
        .bind(bank_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
