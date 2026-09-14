//! 设置项命令：主题 / AI 配置（技术文档 7.2）。
//! API Key 仅 Rust 内部可读全值，前端只拿脱敏值。

use tauri::State;

use crate::common::sqlite_pool;
use crate::models::{AiConfig, AiConfigFull};

/// 读取单个设置项。注意：`ai_api_key` 为敏感键，仅返回脱敏值（防止前端读回完整 Key）。
#[tauri::command]
pub async fn get_setting(
    pool: State<'_, sqlx::SqlitePool>,
    key: String,
) -> Result<Option<String>, String> {
    let pool = sqlite_pool(&pool)?;
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?1")
        .bind(&key)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    if key == "ai_api_key" {
        return Ok(row.map(|r| mask_key(&r.0)));
    }
    Ok(row.map(|r| r.0))
}

/// 写入单个设置项（覆盖）
#[tauri::command]
pub async fn set_setting(
    pool: State<'_, sqlx::SqlitePool>,
    key: String,
    value: String,
) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 读取 AI 配置（Key 脱敏）
#[tauri::command]
pub async fn get_ai_config(pool: State<'_, sqlx::SqlitePool>) -> Result<AiConfig, String> {
    let pool = sqlite_pool(&pool)?;
    let (base_url, api_key, model, judge_enabled, analysis_enabled) = load_ai_settings(pool).await?;
    Ok(AiConfig {
        base_url,
        model,
        api_key_masked: mask_key(&api_key),
        judge_enabled,
        analysis_enabled,
    })
}

/// 保存 AI 配置；api_key 为空表示不修改
#[tauri::command]
pub async fn save_ai_config(
    pool: State<'_, sqlx::SqlitePool>,
    base_url: String,
    api_key: Option<String>,
    model: String,
    judge_enabled: bool,
    analysis_enabled: bool,
) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;
    set_value(&pool, "ai_base_url", base_url.trim()).await?;
    if let Some(key) = api_key {
        let key = key.trim();
        if !key.is_empty() {
            set_value(&pool, "ai_api_key", key).await?;
        }
    }
    set_value(&pool, "ai_model", model.trim()).await?;
    set_value(&pool, "ai_judge_enabled", if judge_enabled { "1" } else { "0" }).await?;
    set_value(
        &pool,
        "ai_analysis_enabled",
        if analysis_enabled { "1" } else { "0" },
    )
    .await?;
    Ok(())
}

async fn set_value(pool: &sqlx::SqlitePool, key: &str, value: &str) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

async fn get_value(pool: &sqlx::SqlitePool, key: &str) -> Result<String, String> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.map(|r| r.0).unwrap_or_default())
}

/// 读取 AI 配置原始值（供 ai 模块使用；含完整 Key，仅内部调用）
pub(crate) async fn load_ai_settings(
    pool: &sqlx::SqlitePool,
) -> Result<(String, String, String, bool, bool), String> {
    let base_url = get_value(pool, "ai_base_url").await?;
    let api_key = get_value(pool, "ai_api_key").await?;
    let model = get_value(pool, "ai_model").await?;
    let judge_enabled = get_value(pool, "ai_judge_enabled").await? == "1";
    let analysis_enabled = get_value(pool, "ai_analysis_enabled").await? == "1";
    Ok((base_url, api_key, model, judge_enabled, analysis_enabled))
}

/// 读取完整 AI 配置（含完整 Key）。仅供前端 JS 直调 axios 使用：
/// 前端在内存中构建客户端，Key 不落 localStorage / sessionStorage。
#[tauri::command]
pub async fn get_ai_config_full(
    pool: State<'_, sqlx::SqlitePool>,
) -> Result<AiConfigFull, String> {
    let pool = sqlite_pool(&pool)?;
    let (base_url, api_key, model, judge_enabled, analysis_enabled) =
        load_ai_settings(pool).await?;
    Ok(AiConfigFull {
        base_url,
        api_key,
        model,
        judge_enabled,
        analysis_enabled,
    })
}

/// Key 脱敏：前 4 后 4；长度不足则全隐藏
fn mask_key(key: &str) -> String {
    let len = key.chars().count();
    if len <= 8 {
        return if key.is_empty() {
            String::new()
        } else {
            "****".into()
        };
    }
    let chars: Vec<char> = key.chars().collect();
    format!(
        "{}{}{}",
        chars[..4].iter().collect::<String>(),
        "****",
        chars[len - 4..].iter().collect::<String>()
    )
}
