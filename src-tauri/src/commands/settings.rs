use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(rename = "roxyExePath")]
    pub roxy_exe_path: Option<String>,
    #[serde(default = "default_auto_detect")]
    pub auto_detect_enabled: bool,
}

fn default_auto_detect() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            roxy_exe_path: None,
            auto_detect_enabled: true,
        }
    }
}

/// 获取配置文件路径
fn get_settings_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "无法获取配置目录".to_string())?;
    
    let app_config_dir = config_dir.join("roxybrowser-manager");
    
    // 确保目录存在
    if !app_config_dir.exists() {
        std::fs::create_dir_all(&app_config_dir)
            .map_err(|e| format!("无法创建配置目录: {}", e))?;
    }
    
    Ok(app_config_dir.join("settings.json"))
}

/// 加载设置
pub fn load_settings() -> Result<AppSettings, String> {
    let settings_path = get_settings_path()?;
    
    if !settings_path.exists() {
        // 如果配置文件不存在，返回默认配置
        return Ok(AppSettings::default());
    }
    
    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("无法读取配置文件: {}", e))?;
    
    let settings: AppSettings = serde_json::from_str(&content)
        .map_err(|e| format!("配置文件格式错误: {}", e))?;
    
    Ok(settings)
}

/// 保存设置
pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let settings_path = get_settings_path()?;
    
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("无法序列化配置: {}", e))?;
    
    std::fs::write(&settings_path, content)
        .map_err(|e| format!("无法写入配置文件: {}", e))?;
    
    Ok(())
}

/// 验证 RoxyBrowser 可执行文件路径
pub fn validate_path(path: &str) -> Result<bool, String> {
    let path_buf = PathBuf::from(path);
    
    // 检查文件是否存在
    if !path_buf.exists() {
        return Ok(false);
    }
    
    // 检查是否是文件
    if !path_buf.is_file() {
        return Ok(false);
    }
    
    // 检查文件名是否为 RoxyBrowser.exe (Windows) 或 RoxyBrowser (其他平台)
    if let Some(file_name) = path_buf.file_name() {
        let name = file_name.to_string_lossy();
        #[cfg(target_os = "windows")]
        {
            if name.to_lowercase() != "roxybrowser.exe" {
                return Ok(false);
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            if name != "RoxyBrowser" {
                return Ok(false);
            }
        }
    } else {
        return Ok(false);
    }
    
    Ok(true)
}

/// 获取增强的默认路径列表
pub fn get_enhanced_default_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        // 优先级1: 用户级安装路径
        if let Some(local_app_data) = dirs::data_local_dir() {
            paths.push(local_app_data.join("Programs").join("RoxyBrowser").join("RoxyBrowser.exe"));
        }
        
        // 优先级2: C盘系统级路径
        paths.push(PathBuf::from(r"C:\Program Files\RoxyBrowser\RoxyBrowser.exe"));
        paths.push(PathBuf::from(r"C:\Program Files (x86)\RoxyBrowser\RoxyBrowser.exe"));
        
        // 优先级3: 其他常见盘符的 Program Files
        for drive in &['D', 'E', 'F'] {
            paths.push(PathBuf::from(format!(r"{}:\Program Files\RoxyBrowser\RoxyBrowser.exe", drive)));
            paths.push(PathBuf::from(format!(r"{}:\Program Files (x86)\RoxyBrowser\RoxyBrowser.exe", drive)));
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Applications/RoxyBrowser.app"));
    }
    
    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/usr/bin/roxybrowser"));
        paths.push(PathBuf::from("/usr/local/bin/roxybrowser"));
    }
    
    paths
}

/// Tauri 命令: 获取当前配置的 RoxyBrowser 可执行文件路径
#[tauri::command]
pub fn get_roxy_exe_path() -> Result<Option<String>, String> {
    let settings = load_settings()?;
    Ok(settings.roxy_exe_path)
}

/// Tauri 命令: 设置 RoxyBrowser 可执行文件路径
#[tauri::command]
pub fn set_roxy_exe_path(path: String) -> Result<(), String> {
    // 验证路径
    if !validate_path(&path)? {
        return Err("无效的 RoxyBrowser 可执行文件路径".to_string());
    }
    
    // 加载现有设置
    let mut settings = load_settings().unwrap_or_default();
    
    // 更新路径
    settings.roxy_exe_path = Some(path);
    
    // 保存设置
    save_settings(&settings)?;
    
    Ok(())
}

/// Tauri 命令: 验证 RoxyBrowser 可执行文件路径
#[tauri::command]
pub fn validate_roxy_exe_path(path: String) -> Result<bool, String> {
    validate_path(&path)
}

/// Tauri 命令: 自动检测 RoxyBrowser 路径
#[tauri::command]
pub fn auto_detect_roxy_path() -> Result<Option<String>, String> {
    let default_paths = get_enhanced_default_paths();
    
    for path in default_paths {
        if path.exists() && path.is_file() {
            return Ok(Some(path.to_string_lossy().to_string()));
        }
    }
    
    Ok(None)
}

/// Tauri 命令: 使用文件对话框选择可执行文件
#[tauri::command]
pub async fn browse_for_exe(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let file_path = app.dialog()
        .file()
        .set_title("选择 RoxyBrowser 可执行文件")
        .add_filter("可执行文件", &["exe"])
        .blocking_pick_file();
    
    if let Some(path) = file_path {
        if let Some(path_ref) = path.as_path() {
            let path_str = path_ref.to_string_lossy().to_string();
            
            // 验证选择的文件
            if validate_path(&path_str)? {
                Ok(Some(path_str))
            } else {
                Err("所选文件不是有效的 RoxyBrowser 可执行文件".to_string())
            }
        } else {
            Err("无法获取文件路径".to_string())
        }
    } else {
        Ok(None)
    }
}

/// Tauri 命令: 清除配置的路径
#[tauri::command]
pub fn clear_roxy_exe_path() -> Result<(), String> {
    let mut settings = load_settings().unwrap_or_default();
    settings.roxy_exe_path = None;
    save_settings(&settings)?;
    Ok(())
}
