//! 统计模块：SQL 聚合计算，返回简单数值给前端渲染（技术文档 5.4）。
//!
//! 判分口径：manual > ai > machine（业务文档 6.5）。
//! 删除题目时 answer_record 级联删除，统计数据自动同步更新（业务文档 7.4）。

use tauri::State;

use crate::common::sqlite_pool;
use crate::models::{BankStats, StatsSummary};

/// 全局统计：总题量、总刷题量、正确数、错误数、整体正确率
#[tauri::command]
pub async fn get_stats(pool: State<'_, sqlx::SqlitePool>) -> Result<StatsSummary, String> {
    let pool = sqlite_pool(&pool)?;
    let row: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT
           (SELECT COUNT(*) FROM question) AS total_questions,
           (SELECT COUNT(*) FROM answer_record) AS total_records,
           (SELECT COUNT(*) FROM answer_record
              WHERE COALESCE(manual_result, ai_result, machine_result) = 1) AS correct_count,
           (SELECT COUNT(*) FROM answer_record
              WHERE COALESCE(manual_result, ai_result, machine_result) = 0) AS wrong_count",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let (total_questions, total_records, correct_count, wrong_count) = row;
    Ok(StatsSummary {
        total_questions,
        total_records,
        correct_count,
        wrong_count,
        correct_rate: calc_rate(correct_count, wrong_count),
    })
}

/// 分题库统计：按 bank_id 分组统计（业务文档 5.1「分题库统计」）
#[tauri::command]
pub async fn get_bank_stats(pool: State<'_, sqlx::SqlitePool>) -> Result<Vec<BankStats>, String> {
    let pool = sqlite_pool(&pool)?;
    let rows: Vec<(i64, String, i64, i64, i64, i64)> = sqlx::query_as(
        "SELECT
           b.id,
           b.name,
           (SELECT COUNT(*) FROM question q WHERE q.bank_id = b.id) AS total_questions,
           (SELECT COUNT(*) FROM answer_record ar
              JOIN question q ON q.id = ar.question_id
              WHERE q.bank_id = b.id) AS total_records,
           (SELECT COUNT(*) FROM answer_record ar
              JOIN question q ON q.id = ar.question_id
              WHERE q.bank_id = b.id AND COALESCE(ar.manual_result, ar.ai_result, ar.machine_result) = 1) AS correct_count,
           (SELECT COUNT(*) FROM answer_record ar
              JOIN question q ON q.id = ar.question_id
              WHERE q.bank_id = b.id AND COALESCE(ar.manual_result, ar.ai_result, ar.machine_result) = 0) AS wrong_count
         FROM question_bank b
         ORDER BY b.id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(bank_id, bank_name, total_questions, total_records, correct_count, wrong_count)| {
            BankStats {
                bank_id,
                bank_name,
                total_questions,
                total_records,
                correct_count,
                wrong_count,
                correct_rate: calc_rate(correct_count, wrong_count),
            }
        })
        .collect())
}

/// 正确率：百分数（0-100），保留两位小数
fn calc_rate(correct: i64, wrong: i64) -> f64 {
    let judged = correct + wrong;
    if judged > 0 {
        ((correct as f64 / judged as f64) * 10000.0).round() / 100.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::calc_rate;

    #[test]
    fn test_calc_rate() {
        assert_eq!(calc_rate(0, 0), 0.0);
        assert_eq!(calc_rate(1, 0), 100.0);
        assert_eq!(calc_rate(0, 1), 0.0);
        assert_eq!(calc_rate(1, 1), 50.0);
        assert_eq!(calc_rate(2, 1), 66.67);
    }
}
