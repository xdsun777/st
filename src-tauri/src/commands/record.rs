//! 做题记录（answer_record）命令：提交作答、错题/收藏标记、错题本与收藏查询。
//!
//! 业务规则（业务文档 4.5 / 4.6 / 7.3）：
//! - single/multi/judge/fill 机器判错 → is_fault=1，fault_count 累加
//! - essay 仅 manual_result=0 置错题；fill 手动覆写结果
//! - 同一题目多次做错，错题本仅存一条（查询按题目去重），fault_count 记录累计做错次数
//! - 移出错题：仅清除 is_fault 标记，做题记录保留
//! - 收藏独立于错题；删除题目时收藏随记录级联删除自动取消

use tauri::State;

use crate::common::{now_ms, resolve_result, row_to_question, sqlite_pool};
use crate::models::{AnswerRecord, Question, SubmitAnswerInput};

/// 提交作答：生成一条做题记录，按业务规则计算错题标记与做错累计次数。
/// 最终对错 = manual_result（优先） ?? machine_result。
#[tauri::command]
pub async fn submit_answer(
    pool: State<'_, sqlx::SqlitePool>,
    input: SubmitAnswerInput,
) -> Result<AnswerRecord, String> {
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 题目必须存在
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM question WHERE id = ?1")
            .bind(input.question_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    if exists.is_none() {
        return Err("题目不存在".into());
    }

    // 该题历史做错累计次数（错题本仅存一条、fault_count 承载累计次数）
    let hist: (i64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(fault_count), 0) FROM answer_record WHERE question_id = ?1",
    )
    .bind(input.question_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let final_result = resolve_result(input.machine_result, input.manual_result);
    let is_fault: i64 = if final_result == Some(0) { 1 } else { 0 };
    let fault_count: i64 = if is_fault == 1 { hist.0 + 1 } else { hist.0 };

    let now = now_ms();
    let id = sqlx::query(
        "INSERT INTO answer_record
           (question_id, user_answer, machine_result, manual_result, is_fault, is_collect, fault_count, finish_time)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
    )
    .bind(input.question_id)
    .bind(input.user_answer.as_deref())
    .bind(input.machine_result)
    .bind(input.manual_result)
    .bind(is_fault)
    .bind(fault_count)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(AnswerRecord {
        id,
        question_id: input.question_id,
        user_answer: input.user_answer,
        machine_result: input.machine_result,
        manual_result: input.manual_result,
        is_fault,
        is_collect: 0,
        fault_count,
        finish_time: now,
    })
}

/// 更新错题标记（错题本「移出错题」走此接口）。
/// 仅更新标记，做题记录与 fault_count 完整保留（业务文档 4.5.3）。
#[tauri::command]
pub async fn update_fault(
    pool: State<'_, sqlx::SqlitePool>,
    question_id: i64,
    is_fault: bool,
) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;
    sqlx::query("UPDATE answer_record SET is_fault = ?1 WHERE question_id = ?2")
        .bind(if is_fault { 1 } else { 0 })
        .bind(question_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 更新手动判分结果（填空覆写、简答手动标记对错，业务文档 4.5.1 / 7.3）。
/// - 更新该题最新一条做题记录的 manual_result（优先级高于 machine_result）
/// - 同步该题全部记录的错题标记 is_fault
/// - 标记为错时，最新记录 fault_count 累加 1
#[tauri::command]
pub async fn update_manual_result(
    pool: State<'_, sqlx::SqlitePool>,
    question_id: i64,
    manual_result: i64,
) -> Result<(), String> {
    if manual_result != 0 && manual_result != 1 {
        return Err("手动判分结果只能为 0（错误）或 1（正确）".into());
    }
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let latest: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM answer_record WHERE question_id = ?1 ORDER BY id DESC LIMIT 1",
    )
    .bind(question_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if let Some((record_id,)) = latest {
        let hist: (i64,) = sqlx::query_as(
            "SELECT COALESCE(MAX(fault_count), 0) FROM answer_record WHERE question_id = ?1",
        )
        .bind(question_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let is_fault = if manual_result == 0 { 1 } else { 0 };
        let fault_count = if manual_result == 0 { hist.0 + 1 } else { hist.0 };

        sqlx::query(
            "UPDATE answer_record SET manual_result = ?1, is_fault = ?2, fault_count = ?3 WHERE id = ?4",
        )
        .bind(manual_result)
        .bind(is_fault)
        .bind(fault_count)
        .bind(record_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // 错题标记为题目级状态：同步该题全部记录
        sqlx::query("UPDATE answer_record SET is_fault = ?1 WHERE question_id = ?2")
            .bind(is_fault)
            .bind(question_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 收藏/取消收藏。收藏独立于错题（业务文档 4.6）。
/// 若题目从未作答，则创建一条仅承载收藏标记的记录；取消收藏后保留空记录。
#[tauri::command]
pub async fn update_collect(
    pool: State<'_, sqlx::SqlitePool>,
    question_id: i64,
    is_collect: bool,
) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM question WHERE id = ?1")
            .bind(question_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    if exists.is_none() {
        return Err("题目不存在".into());
    }

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM answer_record WHERE question_id = ?1",
    )
    .bind(question_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if count.0 == 0 {
        // 无做题记录：插入一条仅承载收藏标记的记录
        sqlx::query(
            "INSERT INTO answer_record
               (question_id, user_answer, machine_result, manual_result, is_fault, is_collect, fault_count, finish_time)
             VALUES (?1, NULL, NULL, NULL, 0, ?2, 0, ?3)",
        )
        .bind(question_id)
        .bind(if is_collect { 1 } else { 0 })
        .bind(now_ms())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        sqlx::query("UPDATE answer_record SET is_collect = ?1 WHERE question_id = ?2")
            .bind(if is_collect { 1 } else { 0 })
            .bind(question_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 错题本：is_fault=1 的题目（按题目去重，仅存一条），支持标签筛选
#[tauri::command]
pub async fn get_fault_questions(
    pool: State<'_, sqlx::SqlitePool>,
    tag_id: Option<i64>,
) -> Result<Vec<Question>, String> {
    let pool = sqlite_pool(&pool)?;
    let rows = sqlx::query(
        "SELECT q.id, q.bank_id, q.q_type, q.content, q.options, q.answer, q.analysis,
                q.create_time, q.update_time,
                COALESCE(GROUP_CONCAT(t.name), '') AS tags
         FROM question q
         LEFT JOIN question_tag qt ON qt.question_id = q.id
         LEFT JOIN tag t ON t.id = qt.tag_id
         WHERE EXISTS (
                 SELECT 1 FROM answer_record ar
                 WHERE ar.question_id = q.id AND ar.is_fault = 1
               )
           AND (?1 IS NULL OR q.id IN (SELECT question_id FROM question_tag WHERE tag_id = ?1))
         GROUP BY q.id
         ORDER BY q.id",
    )
    .bind(tag_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    rows.iter().map(row_to_question).collect()
}

/// 收藏题目列表：is_collect=1 的题目（按题目去重），支持标签筛选
#[tauri::command]
pub async fn get_collect_questions(
    pool: State<'_, sqlx::SqlitePool>,
    tag_id: Option<i64>,
) -> Result<Vec<Question>, String> {
    let pool = sqlite_pool(&pool)?;
    let rows = sqlx::query(
        "SELECT q.id, q.bank_id, q.q_type, q.content, q.options, q.answer, q.analysis,
                q.create_time, q.update_time,
                COALESCE(GROUP_CONCAT(t.name), '') AS tags
         FROM question q
         LEFT JOIN question_tag qt ON qt.question_id = q.id
         LEFT JOIN tag t ON t.id = qt.tag_id
         WHERE EXISTS (
                 SELECT 1 FROM answer_record ar
                 WHERE ar.question_id = q.id AND ar.is_collect = 1
               )
           AND (?1 IS NULL OR q.id IN (SELECT question_id FROM question_tag WHERE tag_id = ?1))
         GROUP BY q.id
         ORDER BY q.id",
    )
    .bind(tag_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    rows.iter().map(row_to_question).collect()
}

/// 某题的做题记录列表（统计/复盘用）
#[tauri::command]
pub async fn get_answer_records(
    pool: State<'_, sqlx::SqlitePool>,
    question_id: i64,
) -> Result<Vec<AnswerRecord>, String> {
    let pool = sqlite_pool(&pool)?;
    let records = sqlx::query_as::<_, AnswerRecord>(
        "SELECT id, question_id, user_answer, machine_result, manual_result,
                is_fault, is_collect, fault_count, finish_time
         FROM answer_record
         WHERE question_id = ?1
         ORDER BY id DESC",
    )
    .bind(question_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(records)
}
