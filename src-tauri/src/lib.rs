//! 应用入口：注册插件（SQLite、文件系统、对话框）与 Tauri Commands。

mod commands;
mod db;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        // 默认启用 GPU 渲染
        // 但如果失败，WebKitGTK 会有错误日志
        // 我们可以通过环境变量允许动态切换
        if should_disable_gpu_rendering() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:st.db", db::migrations())
                .build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_banks,
            commands::create_bank,
            commands::delete_bank,
            commands::get_questions,
            commands::get_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
                if line.starts_with("Gid:") {
                    // 实际运行中的 GID
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(gid) = parts[1].parse::<u32>() {
                            // 这是实际的 GID，但不是所有组
                        }
                    }
                }
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