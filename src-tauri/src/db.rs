//! 数据库初始化：严格按照技术文档「4. 数据库设计」建表。
//!
//! 共六张表：question_bank、question、tag、question_tag、
//! answer_record（同时承载错题、收藏、作答记录）、practice_session。
//!
//! 由 lib.rs 在应用启动时调用 `init_schema` 执行（连接池已启用外键约束，
//! 此处不再需要 PRAGMA foreign_keys）。

/// 建表迁移语句（按顺序执行）
pub const MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS question_bank (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL,
      create_time INTEGER NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS question (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      bank_id INTEGER NOT NULL REFERENCES question_bank(id) ON DELETE CASCADE,
      q_type TEXT NOT NULL,
      content TEXT NOT NULL,
      options TEXT,
      answer TEXT NOT NULL,
      analysis TEXT,
      create_time INTEGER NOT NULL,
      update_time INTEGER NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS tag (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL UNIQUE
    );",
    "CREATE TABLE IF NOT EXISTS question_tag (
      question_id INTEGER NOT NULL REFERENCES question(id) ON DELETE CASCADE,
      tag_id INTEGER NOT NULL REFERENCES tag(id) ON DELETE CASCADE,
      PRIMARY KEY(question_id, tag_id)
    );",
    "CREATE TABLE IF NOT EXISTS answer_record (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      question_id INTEGER NOT NULL REFERENCES question(id) ON DELETE CASCADE,
      user_answer TEXT,
      machine_result INTEGER,
      manual_result INTEGER,
      is_fault INTEGER NOT NULL DEFAULT 0,
      is_collect INTEGER NOT NULL DEFAULT 0,
      fault_count INTEGER NOT NULL DEFAULT 0,
      finish_time INTEGER NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS practice_session (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      bank_id INTEGER,
      tag_filter TEXT,
      current_index INTEGER NOT NULL DEFAULT 0,
      practice_mode TEXT NOT NULL,
      create_time INTEGER NOT NULL
    );",
];

/// 执行建表迁移（幂等，应用启动时调用）
pub async fn init_schema(pool: &sqlx::SqlitePool) -> Result<(), String> {
    for sql in MIGRATIONS {
        sqlx::query(sql)
            .execute(pool)
            .await
            .map_err(|e| format!("建表失败：{e}\nSQL: {sql}"))?;
    }
    Ok(())
}
