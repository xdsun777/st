//! 备份导出 / 恢复（技术文档 5.5 / 6.2）。
//!
//! - 导出：读取全部业务表 → 序列化 JSON → 写入自定义后缀 `.qpbackup` 文件
//! - 恢复：解析校验备份 JSON → 事务内清空全部业务表并写入备份数据
//! - 异常捕获：文件损坏、格式错误时返回错误，原有数据保持不变（先解析后清库）

use tauri::{AppHandle, State};
use tauri_plugin_fs::{FilePath, FsExt};

use crate::common::{now_ms, sqlite_pool};
use crate::models::{
    AiAnalysisRow, AnswerRecord, BackupData, BackupResult, PracticeSession, QuestionBankRow,
    QuestionRow, QuestionTagRow, SettingRow, Tag,
};

const BACKUP_VERSION: u32 = 1;

/// 导出全量备份到指定路径（.qpbackup 自定义格式，不与 CSV 互通）。
/// 使用 tauri-plugin-fs 写入：桌面为普通文件路径，Android 上为
/// ACTION_CREATE_DOCUMENT 返回的 `content://` URI，不能直接走 `std::fs`。
#[tauri::command]
pub async fn export_backup(
    app: AppHandle,
    pool: State<'_, sqlx::SqlitePool>,
    path: FilePath,
) -> Result<BackupResult, String> {
    let path_str = path.to_string();
    if path_str.trim().is_empty() {
        return Err("备份文件路径不能为空".into());
    }
    let pool = sqlite_pool(&pool)?;
    let data = read_all(pool).await?;
    let json = serde_json::to_vec_pretty(&data).map_err(|e| format!("备份数据序列化失败：{e}"))?;
    write_file(&app, path, &json).map_err(|e| format!("备份文件写入失败（{path_str}）：{e}"))?;
    Ok(result_of(&path_str, &data))
}

/// 从备份文件恢复：覆盖本地全部数据（前端已做风险二次确认）。
/// 解析失败或写入失败均不会破坏原有数据。
#[tauri::command]
pub async fn import_backup(
    app: AppHandle,
    pool: State<'_, sqlx::SqlitePool>,
    path: FilePath,
) -> Result<BackupResult, String> {
    let path_str = path.to_string();
    if path_str.trim().is_empty() {
        return Err("备份文件路径不能为空".into());
    }

    // 1. 先完整读取并解析校验；失败则直接返回，不触碰原库。
    //    Android 文件选择器返回的是 content:// URI，必须经 tauri-plugin-fs 读取。
    let raw = app
        .fs()
        .read(path)
        .map_err(|e| format!("备份文件读取失败（{path_str}）：{e}"))?;
    let data: BackupData = serde_json::from_slice(&raw)
        .map_err(|e| format!("备份文件解析失败（格式错误或已损坏）：{e}"))?;
    if data.version != BACKUP_VERSION {
        return Err(format!(
            "备份文件版本不受支持（{}），当前支持版本 {}",
            data.version, BACKUP_VERSION
        ));
    }

    // 2. 事务内清空全部业务表并写入备份数据
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    clear_all(&mut tx).await?;

    for bank in &data.question_banks {
        sqlx::query("INSERT INTO question_bank (id, name, create_time) VALUES (?1, ?2, ?3)")
            .bind(bank.id)
            .bind(&bank.name)
            .bind(bank.create_time)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    for tag in &data.tags {
        sqlx::query("INSERT INTO tag (id, name) VALUES (?1, ?2)")
            .bind(tag.id)
            .bind(&tag.name)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    for q in &data.questions {
        sqlx::query(
            "INSERT INTO question
               (id, bank_id, q_type, content, options, answer, analysis, create_time, update_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(q.id)
        .bind(q.bank_id)
        .bind(&q.q_type)
        .bind(&q.content)
        .bind(q.options.as_deref())
        .bind(&q.answer)
        .bind(q.analysis.as_deref())
        .bind(q.create_time)
        .bind(q.update_time)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    for qt in &data.question_tags {
        sqlx::query("INSERT INTO question_tag (question_id, tag_id) VALUES (?1, ?2)")
            .bind(qt.question_id)
            .bind(qt.tag_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    for ar in &data.answer_records {
        sqlx::query(
            "INSERT INTO answer_record
               (id, question_id, user_answer, machine_result, ai_result, manual_result,
                is_fault, is_collect, fault_count, finish_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .bind(ar.id)
        .bind(ar.question_id)
        .bind(ar.user_answer.as_deref())
        .bind(ar.machine_result)
        .bind(ar.ai_result)
        .bind(ar.manual_result)
        .bind(ar.is_fault)
        .bind(ar.is_collect)
        .bind(ar.fault_count)
        .bind(ar.finish_time)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    for ps in &data.practice_sessions {
        sqlx::query(
            "INSERT INTO practice_session
               (id, bank_id, tag_filter, current_index, practice_mode, create_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(ps.id)
        .bind(ps.bank_id)
        .bind(ps.tag_filter.as_deref())
        .bind(ps.current_index)
        .bind(&ps.practice_mode)
        .bind(ps.create_time)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    for s in &data.settings {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(&s.key)
        .bind(&s.value)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    for a in &data.ai_analysis {
        sqlx::query(
            "INSERT OR REPLACE INTO ai_analysis_cache (question_id, answer_hash, analysis, create_time)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(a.question_id)
        .bind(&a.answer_hash)
        .bind(&a.analysis)
        .bind(a.create_time)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 3. 全部成功才提交；任一步失败自动回滚，原库保持完整
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(result_of(&path_str, &data))
}

/// 经 tauri-plugin-fs 写文件：桌面写入普通路径；Android 写入 content:// URI。
fn write_file(app: &AppHandle, path: FilePath, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    let mut opts = tauri_plugin_fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    let mut file = app.fs().open(path, opts)?;
    file.write_all(data)?;
    file.flush()
}

/// 读取全部业务表组装备份数据
async fn read_all(pool: &sqlx::SqlitePool) -> Result<BackupData, String> {
    let question_banks = sqlx::query_as::<_, QuestionBankRow>(
        "SELECT id, name, create_time FROM question_bank ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let questions = sqlx::query_as::<_, QuestionRow>(
        "SELECT id, bank_id, q_type, content, options, answer, analysis, create_time, update_time
         FROM question ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let tags = sqlx::query_as::<_, Tag>("SELECT id, name FROM tag ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let question_tags =
        sqlx::query_as::<_, QuestionTagRow>("SELECT question_id, tag_id FROM question_tag")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

    let answer_records = sqlx::query_as::<_, AnswerRecord>(
        "SELECT id, question_id, user_answer, machine_result, ai_result, manual_result,
                is_fault, is_collect, fault_count, finish_time
         FROM answer_record ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let practice_sessions = sqlx::query_as::<_, PracticeSession>(
        "SELECT id, bank_id, tag_filter, current_index, practice_mode, create_time
         FROM practice_session ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let settings = sqlx::query_as::<_, SettingRow>("SELECT key, value FROM settings ORDER BY key")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let ai_analysis = sqlx::query_as::<_, AiAnalysisRow>(
        "SELECT question_id, answer_hash, analysis, create_time
         FROM ai_analysis_cache ORDER BY question_id, answer_hash",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(BackupData {
        version: BACKUP_VERSION,
        exported_at: now_ms(),
        question_banks,
        questions,
        tags,
        question_tags,
        answer_records,
        practice_sessions,
        settings,
        ai_analysis,
    })
}

/// 清空全部业务表（恢复导入前调用）。顺序：先删引用表，再删主表。
async fn clear_all(tx: &mut sqlx::SqliteConnection) -> Result<(), String> {
    sqlx::query("DELETE FROM answer_record")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM ai_analysis_cache")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM question_tag")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM question")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM practice_session")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM settings")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM question_bank")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM tag")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn result_of(path: &str, data: &BackupData) -> BackupResult {
    BackupResult {
        path: path.to_string(),
        question_banks: data.question_banks.len(),
        questions: data.questions.len(),
        tags: data.tags.len(),
        question_tags: data.question_tags.len(),
        answer_records: data.answer_records.len(),
        practice_sessions: data.practice_sessions.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> BackupData {
        BackupData {
            version: BACKUP_VERSION,
            exported_at: 1_700_000_000_000,
            question_banks: vec![QuestionBankRow {
                id: 1,
                name: "测试题库".into(),
                create_time: 1,
            }],
            questions: vec![QuestionRow {
                id: 1,
                bank_id: 1,
                q_type: "single".into(),
                content: "1+1=?".into(),
                options: Some("[\"A.1\",\"B.2\"]".into()),
                answer: "B".into(),
                analysis: Some("基础加法".into()),
                create_time: 1,
                update_time: 1,
            }],
            tags: vec![Tag {
                id: 1,
                name: "数学".into(),
            }],
            question_tags: vec![QuestionTagRow {
                question_id: 1,
                tag_id: 1,
            }],
            answer_records: vec![AnswerRecord {
                id: 1,
                question_id: 1,
                user_answer: Some("B".into()),
                machine_result: Some(1),
                ai_result: None,
                manual_result: None,
                is_fault: 0,
                is_collect: 1,
                fault_count: 0,
                finish_time: 2,
            }],
            practice_sessions: vec![PracticeSession {
                id: 1,
                bank_id: Some(1),
                tag_filter: None,
                current_index: 3,
                practice_mode: "order".into(),
                create_time: 3,
            }],
            settings: vec![],
            ai_analysis: vec![],
        }
    }

    #[test]
    fn test_backup_roundtrip() {
        let data = sample_data();
        let json = serde_json::to_vec_pretty(&data).unwrap();
        let restored: BackupData = serde_json::from_slice(&json).unwrap();
        assert_eq!(restored.version, BACKUP_VERSION);
        assert_eq!(restored.questions.len(), 1);
        assert_eq!(restored.questions[0].content, "1+1=?");
        assert_eq!(restored.answer_records[0].machine_result, Some(1));
        assert_eq!(restored.practice_sessions[0].practice_mode, "order");
    }

    #[test]
    fn test_invalid_json_rejected() {
        let bad: Result<BackupData, _> = serde_json::from_slice(b"not a json");
        assert!(bad.is_err());
    }
}
