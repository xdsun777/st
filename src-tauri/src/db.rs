//! 数据库初始化：严格按照技术文档「4. 数据库设计」建表。
//!
//! 共六张表：question_bank、question、tag、question_tag、
//! answer_record（同时承载错题、收藏、作答记录）、practice_session。
//! 每个语句拆分为独立 migration，避免多语句执行兼容性问题。

use tauri_plugin_sql::{Migration, MigrationKind};

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "enable_foreign_keys",
            sql: "PRAGMA foreign_keys = ON;",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_question_bank",
            sql: "CREATE TABLE IF NOT EXISTS question_bank (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              name TEXT NOT NULL,
              create_time INTEGER NOT NULL
            );",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "create_question",
            sql: "CREATE TABLE IF NOT EXISTS question (
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
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "create_tag",
            sql: "CREATE TABLE IF NOT EXISTS tag (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              name TEXT NOT NULL UNIQUE
            );",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "create_question_tag",
            sql: "CREATE TABLE IF NOT EXISTS question_tag (
              question_id INTEGER NOT NULL REFERENCES question(id) ON DELETE CASCADE,
              tag_id INTEGER NOT NULL REFERENCES tag(id) ON DELETE CASCADE,
              PRIMARY KEY(question_id, tag_id)
            );",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "create_answer_record",
            sql: "CREATE TABLE IF NOT EXISTS answer_record (
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
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "create_practice_session",
            sql: "CREATE TABLE IF NOT EXISTS practice_session (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              bank_id INTEGER,
              tag_filter TEXT,
              current_index INTEGER NOT NULL DEFAULT 0,
              practice_mode TEXT NOT NULL,
              create_time INTEGER NOT NULL
            );",
            kind: MigrationKind::Up,
        },
    ]
}
