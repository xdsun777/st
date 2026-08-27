//! 标签（tag）与题目-标签关联（question_tag）命令。
//!
//! 业务规则（业务文档 4.4）：
//! - 标签依附题目，一题可多标签
//! - 删除标签仅清除题目标记，不删除题目

use tauri::State;
use tauri_plugin_sql::DbPool;

use crate::common::{is_valid_tag_name, sqlite_pool};
use crate::models::Tag;

/// 标签列表
#[tauri::command]
pub async fn list_tags(pool: State<'_, DbPool>) -> Result<Vec<Tag>, String> {
    let pool = sqlite_pool(&pool)?;
    let tags = sqlx::query_as::<_, Tag>("SELECT id, name FROM tag ORDER BY name")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(tags)
}

/// 新建标签：同名标签直接返回已有（tag.name UNIQUE）
#[tauri::command]
pub async fn create_tag(pool: State<'_, DbPool>, name: String) -> Result<Tag, String> {
    let name = name.trim();
    if !is_valid_tag_name(name) {
        return Err("标签名不能为空，且不能包含逗号或分号".into());
    }
    let pool = sqlite_pool(&pool)?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let id = ensure_tag(&mut *conn, name).await?;
    Ok(Tag {
        id,
        name: name.to_string(),
    })
}

/// 删除标签：清除题目上的该标签标记，不删除题目
#[tauri::command]
pub async fn delete_tag(pool: State<'_, DbPool>, tag_id: i64) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM question_tag WHERE tag_id = ?1")
        .bind(tag_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM tag WHERE id = ?1")
        .bind(tag_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 确保标签存在并返回 id（不存在则创建）。供题目创建/批量导入/打标签复用。
pub(crate) async fn ensure_tag(
    conn: &mut sqlx::SqliteConnection,
    name: &str,
) -> Result<i64, String> {
    let name = name.trim();
    sqlx::query("INSERT INTO tag (name) VALUES (?1) ON CONFLICT(name) DO NOTHING")
        .bind(name)
        .execute(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    let row: (i64,) = sqlx::query_as("SELECT id FROM tag WHERE name = ?1")
        .bind(name)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.0)
}

/// 覆盖题目的标签集合（先清空关联，再按入参重建）。
/// 非法标签名会被忽略，其余正常写入。
pub(crate) async fn set_question_tags(
    conn: &mut sqlx::SqliteConnection,
    question_id: i64,
    tags: &[String],
) -> Result<(), String> {
    sqlx::query("DELETE FROM question_tag WHERE question_id = ?1")
        .bind(question_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;

    let mut seen = std::collections::HashSet::new();
    for raw in tags {
        let name = raw.trim();
        if !is_valid_tag_name(name) || !seen.insert(name.to_string()) {
            continue;
        }
        let tag_id = ensure_tag(conn, name).await?;
        sqlx::query("INSERT OR IGNORE INTO question_tag (question_id, tag_id) VALUES (?1, ?2)")
            .bind(question_id)
            .bind(tag_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
