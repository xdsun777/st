//! 题目（question）CRUD 与批量导入命令。
//!
//! 业务规则（业务文档 4.2）：
//! - 单题 CRUD；一道题目仅归属一个题库集，可附加多个标签
//! - CSV 批量导入：题干为空/类型非法/答案格式错误跳过；重复题干保留原题不重复新增
//! - 修改原题题干/答案不清除历史错题、做题记录

use tauri::State;

use crate::commands::tag::set_question_tags;
use crate::common::{is_valid_q_type, now_ms, row_to_question, sqlite_pool};
use crate::models::{BatchInsertResult, NewQuestion, Question, QuestionInput};

/// 题目列表：支持按题库集、按标签筛选；返回题目及其标签名列表
#[tauri::command]
pub async fn get_questions(
    pool: State<'_, sqlx::SqlitePool>,
    bank_id: Option<i64>,
    tag_id: Option<i64>,
) -> Result<Vec<Question>, String> {
    let pool = sqlite_pool(&pool)?;
    let rows = sqlx::query(
        "SELECT q.id, q.bank_id, q.q_type, q.content, q.options, q.answer, q.analysis,
                q.create_time, q.update_time,
                COALESCE(GROUP_CONCAT(t.name), '') AS tags,
                COALESCE((SELECT MAX(is_collect) FROM answer_record WHERE question_id = q.id), 0) AS is_collect
         FROM question q
         LEFT JOIN question_tag qt ON qt.question_id = q.id
         LEFT JOIN tag t ON t.id = qt.tag_id
         WHERE (?1 IS NULL OR q.bank_id = ?1)
           AND (?2 IS NULL OR q.id IN (SELECT question_id FROM question_tag WHERE tag_id = ?2))
         GROUP BY q.id
         ORDER BY q.id",
    )
    .bind(bank_id)
    .bind(tag_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    rows.iter().map(row_to_question).collect()
}

/// 单个题目
#[tauri::command]
pub async fn get_question(pool: State<'_, sqlx::SqlitePool>, question_id: i64) -> Result<Question, String> {
    let pool = sqlite_pool(&pool)?;
    let row = sqlx::query(
        "SELECT q.id, q.bank_id, q.q_type, q.content, q.options, q.answer, q.analysis,
                q.create_time, q.update_time,
                COALESCE(GROUP_CONCAT(t.name), '') AS tags,
                COALESCE((SELECT MAX(is_collect) FROM answer_record WHERE question_id = q.id), 0) AS is_collect
         FROM question q
         LEFT JOIN question_tag qt ON qt.question_id = q.id
         LEFT JOIN tag t ON t.id = qt.tag_id
         WHERE q.id = ?1
         GROUP BY q.id",
    )
    .bind(question_id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    row_to_question(&row)
}

/// 新增单道题目（含标签关联，事务）
#[tauri::command]
pub async fn create_question(
    pool: State<'_, sqlx::SqlitePool>,
    input: QuestionInput,
) -> Result<Question, String> {
    validate_input(&input)?;
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let now = now_ms();
    let id = sqlx::query(
        "INSERT INTO question (bank_id, q_type, content, options, answer, analysis, create_time, update_time)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
    )
    .bind(input.bank_id)
    .bind(input.q_type.as_str())
    .bind(input.content.trim())
    .bind(input.options.as_deref())
    .bind(input.answer.trim())
    .bind(input.analysis.as_deref())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    set_question_tags(&mut *tx, id, &input.tags).await?;
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(Question {
        id,
        bank_id: input.bank_id,
        q_type: input.q_type,
        content: input.content.trim().to_string(),
        options: input.options,
        answer: input.answer.trim().to_string(),
        analysis: input.analysis,
        create_time: now,
        update_time: now,
        tags: clean_tags(&input.tags),
        is_collect: 0,
    })
}

/// 更新题目（含标签关联，事务）。修改题干/答案不清除历史做题记录（业务文档 4.5.4）。
#[tauri::command]
pub async fn update_question(
    pool: State<'_, sqlx::SqlitePool>,
    question_id: i64,
    input: QuestionInput,
) -> Result<Question, String> {
    validate_input(&input)?;
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let now = now_ms();
    let result = sqlx::query(
        "UPDATE question
         SET bank_id = ?1, q_type = ?2, content = ?3, options = ?4, answer = ?5,
             analysis = ?6, update_time = ?7
         WHERE id = ?8",
    )
    .bind(input.bank_id)
    .bind(input.q_type.as_str())
    .bind(input.content.trim())
    .bind(input.options.as_deref())
    .bind(input.answer.trim())
    .bind(input.analysis.as_deref())
    .bind(now)
    .bind(question_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("题目不存在".into());
    }

    set_question_tags(&mut *tx, question_id, &input.tags).await?;
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(Question {
        id: question_id,
        bank_id: input.bank_id,
        q_type: input.q_type,
        content: input.content.trim().to_string(),
        options: input.options,
        answer: input.answer.trim().to_string(),
        analysis: input.analysis,
        create_time: 0, // 由查询接口返回真实值；此处仅作为确认
        update_time: now,
        tags: clean_tags(&input.tags),
        is_collect: 0,
    })
}

/// 删除题目：显式删除做题记录、标签关联与题目本身（收藏随记录删除自动取消，业务文档 4.6）
#[tauri::command]
pub async fn delete_question(pool: State<'_, sqlx::SqlitePool>, question_id: i64) -> Result<(), String> {
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM answer_record WHERE question_id = ?1")
        .bind(question_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM question_tag WHERE question_id = ?1")
        .bind(question_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM question WHERE id = ?1")
        .bind(question_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// CSV 批量导入题目（事务）。逐行校验：
/// - 题型非法、题干为空、答案为空 → 跳过并记录原因
/// - 行带「所属题库集」列 → 按名称创建/复用题库集并归入；否则归入默认 bank_id
/// - 同一题库集内题干重复 → 保留原有题目，跳过并记录原因（业务文档 4.2.3）
#[tauri::command]
pub async fn batch_insert_questions(
    pool: State<'_, sqlx::SqlitePool>,
    bank_id: i64,
    questions: Vec<NewQuestion>,
) -> Result<BatchInsertResult, String> {
    let pool = sqlite_pool(&pool)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 默认题库集是否有效（仅当某行未指定「所属题库集」时需要）
    let default_bank_exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM question_bank WHERE id = ?1")
            .bind(bank_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

    let mut inserted: i64 = 0;
    let mut skipped: i64 = 0;
    let mut skipped_details: Vec<String> = Vec::new();

    for (index, q) in questions.iter().enumerate() {
        let line_no = index + 1;

        if !is_valid_q_type(q.q_type.as_str()) {
            skipped += 1;
            skipped_details.push(format!("第{}行：题型非法（{}）", line_no, q.q_type));
            continue;
        }
        let content = q.content.trim();
        if content.is_empty() {
            skipped += 1;
            skipped_details.push(format!("第{}行：题干为空", line_no));
            continue;
        }
        let answer = q.answer.trim();
        if answer.is_empty() {
            skipped += 1;
            skipped_details.push(format!("第{}行：答案为空", line_no));
            continue;
        }

        // 确定归属题库集：优先用行的「所属题库集」，否则用默认题库集
        let bank_name = q.bank_name.as_deref().map(str::trim).filter(|s| !s.is_empty());
        let row_bank_id: i64 = match bank_name {
            Some(name) => ensure_bank(&mut tx, name).await?,
            None => {
                if default_bank_exists.is_some() {
                    bank_id
                } else {
                    skipped += 1;
                    skipped_details.push(format!(
                        "第{}行：未指定所属题库集，且默认题库集不存在",
                        line_no
                    ));
                    continue;
                }
            }
        };

        // 重复题干：保留原有题目，不重复新增
        let dup: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM question WHERE bank_id = ?1 AND content = ?2 LIMIT 1",
        )
        .bind(row_bank_id)
        .bind(content)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        if dup.is_some() {
            skipped += 1;
            skipped_details.push(format!("第{}行：题干重复，已保留原题", line_no));
            continue;
        }

        let now = now_ms();
        let id = sqlx::query(
            "INSERT INTO question (bank_id, q_type, content, options, answer, analysis, create_time, update_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        )
        .bind(row_bank_id)
        .bind(q.q_type.as_str())
        .bind(content)
        .bind(q.options.as_deref())
        .bind(answer)
        .bind(q.analysis.as_deref())
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();

        set_question_tags(&mut *tx, id, &q.tags).await?;
        inserted += 1;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(BatchInsertResult {
        inserted,
        skipped,
        skipped_details,
    })
}

/// 按名称查找/创建题库集（事务内），返回题库集 id
async fn ensure_bank(conn: &mut sqlx::SqliteConnection, name: &str) -> Result<i64, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("题库集名称不能为空".into());
    }
    if let Some((id,)) = sqlx::query_as("SELECT id FROM question_bank WHERE name = ?1")
        .bind(name)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(id);
    }
    let id = sqlx::query("INSERT INTO question_bank (name, create_time) VALUES (?1, ?2)")
        .bind(name)
        .bind(now_ms())
        .execute(&mut *conn)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();
    Ok(id)
}

fn validate_input(input: &QuestionInput) -> Result<(), String> {
    if !is_valid_q_type(input.q_type.as_str()) {
        return Err(format!("题型非法（{}）", input.q_type));
    }
    if input.content.trim().is_empty() {
        return Err("题干不能为空".into());
    }
    if input.answer.trim().is_empty() {
        return Err("答案不能为空".into());
    }
    Ok(())
}

/// 入参标签去重、去空白、过滤非法
fn clean_tags(tags: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for raw in tags {
        let name = raw.trim();
        if crate::common::is_valid_tag_name(name) && seen.insert(name.to_string()) {
            result.push(name.to_string());
        }
    }
    result
}
