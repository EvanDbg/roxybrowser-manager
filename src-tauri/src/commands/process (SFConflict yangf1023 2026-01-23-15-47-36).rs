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
        // Windows: 尝试从常见路径启动
        let paths = [
            // 用户级安装路径 (实际路径)
            &format!(r"{}\..\Local\Programs\RoxyBrowser\RoxyBrowser.exe", 
                dirs::data_dir().unwrap().display()),
            r"C:\Program Files\RoxyBrowser\RoxyBrowser.exe",
            r"C:\Program Files (x86)\RoxyBrowser\RoxyBrowser.exe",
        ];
        
        for path in &paths {
            let path_buf = std::path::PathBuf::from(path);
            if path_buf.exists() {
                Command::new(&path_buf)
                    .spawn()
                    .map_err(|e| format!("无法启动 RoxyBrowser: {}", e))?;
                return Ok(());
            }
        }
        
        // 备用方案：使用 dirs crate 动态获取用户路径
        if let Some(local_app_data) = dirs::data_local_dir() {
            let user_path = local_app_data.join("Programs").join("RoxyBrowser").join("RoxyBrowser.exe");
            if user_path.exists() {
                Command::new(&user_path)
                    .spawn()
                    .map_err(|e| format!("无法启动 RoxyBrowser: {}", e))?;
                return Ok(());
            }
        }
        
        return Err("未找到 RoxyBrowser 安装路径".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("roxybrowser")
            .spawn()
            .map_err(|e| format!("无法启动 RoxyBrowser: {}", e))?;
        return Ok(());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        return Err("不支持的操作系统".to_string());
    }
}

/// 停止 RoxyBrowser
#[tauri::command]
pub fn stop_roxy() -> Result<(), String> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let mut found = false;
    for (_pid, process) in sys.processes() {
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
