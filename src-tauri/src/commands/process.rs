use serde::Serialize;
use sysinfo::System;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct RoxyStatus {
    #[serde(rename = "isRunning")]
    pub is_running: bool,
    pub pid: Option<u32>,
}

const ROXY_PROCESS_NAME: &str = "RoxyBrowser";

/// 获取 RoxyBrowser 运行状态
#[tauri::command]
pub fn get_roxy_status() -> RoxyStatus {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy();
        if name.contains(ROXY_PROCESS_NAME) {
            return RoxyStatus {
                is_running: true,
                pid: Some(pid.as_u32()),
            };
        }
    }

    RoxyStatus {
        is_running: false,
        pid: None,
    }
}

/// 启动 RoxyBrowser
#[tauri::command]
pub fn start_roxy() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg("RoxyBrowser")
            .spawn()
            .map_err(|e| format!("无法启动 RoxyBrowser: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        use crate::commands::settings::{load_settings, get_enhanced_default_paths};
        
        // 优先级1: 使用用户配置的自定义路径
        if let Ok(settings) = load_settings() {
            if let Some(custom_path) = settings.roxy_exe_path {
                let path_buf = std::path::PathBuf::from(&custom_path);
                if path_buf.exists() {
                    Command::new(&path_buf)
                        .spawn()
                        .map_err(|e| format!("无法启动 RoxyBrowser: {}", e))?;
                    return Ok(());
                }
            }
        }
        
        // 优先级2: 回退到增强的默认路径检测
        let default_paths = get_enhanced_default_paths();
        for path in &default_paths {
            if path.exists() {
                Command::new(&path)
                    .spawn()
                    .map_err(|e| format!("无法启动 RoxyBrowser: {}", e))?;
                return Ok(());
            }
        }
        
        // 未找到任何有效路径
        return Err("未找到 RoxyBrowser 安装路径。\n\n可能的原因：\n• RoxyBrowser 未安装在默认位置\n• 需要手动配置安装路径\n\n请在设置中手动指定可执行文件位置。".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("roxybrowser")
            .spawn()
            .map_err(|e| format!("无法启动 RoxyBrowser: {}", e))?;
    }

    Ok(())
}

/// 停止 RoxyBrowser
#[tauri::command]
pub fn stop_roxy() -> Result<(), String> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let mut found = false;
    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy();
        if name.contains(ROXY_PROCESS_NAME) {
            process.kill();
            found = true;
        }
    }

    if found {
        // 等待进程完全退出
        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(())
    } else {
        Ok(()) // 如果没有运行的进程，也返回成功
    }
}
