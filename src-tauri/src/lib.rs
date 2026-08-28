//! 应用入口：初始化 SQLite 连接池与建表迁移，注册插件与 Tauri Commands。

mod commands;
mod common;
mod db;
mod models;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        // 默认启用 GPU 渲染；若当前用户无 render 权限，禁用硬件加速避免白屏
        if should_disable_gpu_rendering() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }

    tauri::Builder::default()
        .setup(|app| {
            let pool = init_db(app.handle())?;
            app.manage(pool);
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // 题库集
            commands::bank::get_banks,
            commands::bank::create_bank,
            commands::bank::rename_bank,
            commands::bank::delete_bank,
            // 题目与批量导入
            commands::question::get_questions,
            commands::question::get_question,
            commands::question::create_question,
            commands::question::update_question,
            commands::question::delete_question,
            commands::question::batch_insert_questions,
            // 标签
            commands::tag::list_tags,
            commands::tag::create_tag,
            commands::tag::delete_tag,
            // 刷题会话
            commands::practice::save_practice_session,
            commands::practice::load_practice_session,
            commands::practice::clear_practice_session,
            // 做题记录 / 错题 / 收藏
            commands::record::submit_answer,
            commands::record::update_fault,
            commands::record::update_manual_result,
            commands::record::update_collect,
            commands::record::get_fault_questions,
            commands::record::get_collect_questions,
            commands::record::get_answer_records,
            // 统计
            commands::stats::get_stats,
            commands::stats::get_bank_stats,
            // 备份
            commands::backup::export_backup,
            commands::backup::import_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 创建 SQLite 连接池（应用配置目录下 st.db）并执行建表迁移。
fn init_db(app: &tauri::AppHandle) -> Result<sqlx::SqlitePool, Box<dyn std::error::Error>> {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    let config_dir = app.path().app_config_dir()?;
    std::fs::create_dir_all(&config_dir)?;
    let db_path = config_dir.join("st.db");

    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = tauri::async_runtime::block_on(
        SqlitePoolOptions::new().max_connections(5).connect_with(options),
    )?;

    tauri::async_runtime::block_on(db::init_schema(&pool))?;
    Ok(pool)
}

#[cfg(target_os = "linux")]
fn should_disable_gpu_rendering() -> bool {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    // 1. 检查用户是否在 render 组中（最可靠的方式）
    if let Ok(file) = File::open("/proc/self/status") {
        let reader = BufReader::new(file);
        let mut gids = Vec::new();

        for line in reader.lines() {
            if let Ok(line) = line {
                if line.starts_with("Groups:") {
                    gids = line
                        .split_whitespace()
                        .skip(1)
                        .filter_map(|s| s.parse::<u32>().ok())
                        .collect();
                    break;
                }
            }
        }

        // 查找 render 组的 GID
        if let Ok(file) = File::open("/etc/group") {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 3 && parts[0] == "render" {
                        if let Ok(render_gid) = parts[2].parse::<u32>() {
                            if !gids.contains(&render_gid) {
                                // 不在 render 组中，禁用 GPU 渲染
                                eprintln!("⚠️ 用户不在 render 组中，禁用 GPU 渲染");
                                return true;
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    // 2. 作为备用，使用 id 命令
    if let Ok(output) = std::process::Command::new("id").args(["-Gn"]).output() {
        if let Ok(groups) = String::from_utf8(output.stdout) {
            if !groups.split_whitespace().any(|g| g == "render" || g == "video") {
                eprintln!("⚠️ 用户不在 render/video 组中，禁用 GPU 渲染");
                return true;
            }
        }
    }

    eprintln!("✅ GPU 渲染可用，启用硬件加速");
    false // 启用 GPU 渲染
}
