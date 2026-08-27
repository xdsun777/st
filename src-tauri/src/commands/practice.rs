//! 刷题会话（practice_session）读写命令。
//!
//! 业务规则（业务文档 7.3 / 技术文档 4）：
//! - 中途退出保存当前会话进度；再次进入可继续会话或重置会话
//! - practice_session 只保留一份有效会话，新建刷题时覆盖旧会话

use tauri::State;
use tauri_plugin_sql::DbPool;

use crate::common::{now_ms, sqlite_pool};
use crate::models::PracticeSession;

const VALID_MODES: [&str; 2] = ["order", "random"];

/// 保存刷题会话（覆盖旧会话）
#[tauri::command]
pub async fn save_practice_session(
    pool: State<'_, DbPool>,
    bank_id: Option<i64>,
    tag_filter: Option<String>,
    current_index: i64,
    practice_mode: String,
) -> Result<PracticeSession, String> {
    if !VALID_MODES.contains(&practice_mode.as_str()) {
        return Err(format!("刷题模式非法（{}），仅支持 order / random", practice_mode));
    }
    if current_index < 0 {
        return Err("刷题进度不能为负数".into());
    }

    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 只保留一份有效会话：新建刷题时覆盖旧会话
    sqlx::query("DELETE FROM practice_session")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let now = now_ms();
    let id = sqlx::query(
        "INSERT INTO practice_session (bank_id, tag_filter, current_index, practice_mode, create_time)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(bank_id)
    .bind(tag_filter.as_deref())
    .bind(current_index)
    .bind(practice_mode.as_str())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(PracticeSession {
        id,
        bank_id,
        tag_filter,
        current_index,
        practice_mode,
        create_time: now,
    })
}

/// 加载当前刷题会话（无会话返回 null）
#[tauri::command]
pub async fn load_practice_session(
    pool: State<'_, DbPool>,
) -> Result<Option<PracticeSession>, String> {
    let pool = sqlite_pool(&pool)?;
    let session = sqlx::query_as::<_, PracticeSession>(
        "SELECT id, bank_id, tag_filter, current_index, practice_mode, create_time
         FROM practice_session
         ORDER BY id DESC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(session)
}

/// 清除刷题会话（切换/新建刷题时旧进度作废）
#[tauri::command]
pub async fn clear_practice_session(pool: State<'_, DbPool>) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;
    sqlx::query("DELETE FROM practice_session")
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
