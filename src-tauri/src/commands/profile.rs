use crate::models::user::{AppConfig, UserProfile};
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// RoxyBrowser 数据目录
fn get_roxy_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap()
            .join("Library/Application Support/RoxyBrowser")
    }
    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 AppData\Roaming (而非 Local)
        dirs::data_dir()
            .unwrap()
            .join("RoxyBrowser")
    }
    #[cfg(target_os = "linux")]
    {
        dirs::config_dir()
            .unwrap()
            .join("RoxyBrowser")
    }
}

/// 应用配置目录
fn get_app_config_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".roxy_manager")
}

/// 用户配置文件备份目录
fn get_profiles_dir() -> PathBuf {
    get_app_config_dir().join("profiles")
}

/// 应用配置文件路径
fn get_config_path() -> PathBuf {
    get_app_config_dir().join("config.json")
}

/// 需要备份的文件和目录列表
const BACKUP_ITEMS: &[&str] = &[
    "config.json",
    "Local Storage",
    "Cookies",
    "Session Storage",
    "IndexedDB",
];

/// 加载应用配置
fn load_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

/// 保存应用配置
fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_dir = get_app_config_dir();
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("无法创建配置目录: {}", e))?;
    
    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("无法序列化配置: {}", e))?;
    
    fs::write(&config_path, content)
        .map_err(|e| format!("无法保存配置: {}", e))?;
    
    Ok(())
}

/// 备份 RoxyBrowser 数据到指定用户目录
fn backup_roxy_data(email: &str) -> Result<(), String> {
    let roxy_dir = get_roxy_data_dir();
    let profile_dir = get_profiles_dir().join(email);
    
    fs::create_dir_all(&profile_dir)
        .map_err(|e| format!("无法创建用户配置目录: {}", e))?;
    
    for item in BACKUP_ITEMS {
        let src = roxy_dir.join(item);
        let dst = profile_dir.join(item);
        
        if src.exists() {
            if src.is_dir() {
                copy_dir_all(&src, &dst)?;
            } else {
                fs::copy(&src, &dst)
                    .map_err(|e| format!("无法复制 {}: {}", item, e))?;
            }
        }
    }
    
    Ok(())
}

/// 恢复用户数据到 RoxyBrowser 目录
fn restore_roxy_data(email: &str) -> Result<(), String> {
    let roxy_dir = get_roxy_data_dir();
    let profile_dir = get_profiles_dir().join(email);
    
    if !profile_dir.exists() {
        return Err(format!("未找到用户配置: {}", email));
    }
    
    for item in BACKUP_ITEMS {
        let src = profile_dir.join(item);
        let dst = roxy_dir.join(item);
        
        // 先删除目标
        if dst.exists() {
            if dst.is_dir() {
                fs::remove_dir_all(&dst).ok();
            } else {
                fs::remove_file(&dst).ok();
            }
        }
        
        // 复制源
        if src.exists() {
            if src.is_dir() {
                copy_dir_all(&src, &dst)?;
            } else {
                fs::copy(&src, &dst)
                    .map_err(|e| format!("无法复制 {}: {}", item, e))?;
            }
        }
    }
    
    Ok(())
}

/// 递归复制目录
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("无法创建目录 {:?}: {}", dst, e))?;
    
    for entry in fs::read_dir(src).map_err(|e| format!("无法读取目录: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let ty = entry.file_type().map_err(|e| format!("获取文件类型失败: {}", e))?;
        
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("复制文件失败: {}", e))?;
        }
    }
    
    Ok(())
}

/// 从 RoxyBrowser 本地存储中读取当前用户邮箱
fn read_current_email_from_roxy() -> Option<String> {
    let roxy_dir = get_roxy_data_dir();
    let local_storage = roxy_dir.join("Local Storage/leveldb");
    
    if !local_storage.exists() {
        return None;
    }
    
    // 读取 .log 文件查找邮箱 (RoxyBrowser 存储用户信息在 log 文件中)
    if let Ok(entries) = fs::read_dir(&local_storage) {
        for entry in entries.flatten() {
            let path = entry.path();
            // 检查 .log 文件（主要数据）和 .ldb 文件（备用）
            let ext = path.extension().map(|e| e.to_string_lossy().to_string());
            if ext.as_deref() == Some("log") || ext.as_deref() == Some("ldb") {
                if let Ok(content) = fs::read(&path) {
                    let content_str = String::from_utf8_lossy(&content);
                    if let Some(email) = extract_email(&content_str) {
                        return Some(email);
                    }
                }
            }
        }
    }
    
    None
}

/// 从内容中提取邮箱地址
fn extract_email(content: &str) -> Option<String> {
    // 尝试匹配 "userInfo":{"email":"xxx@xxx.com"} 格式
    if let Some(pos) = content.find("\"userInfo\":{\"email\":\"") {
        let start = pos + 21;
        if let Some(end) = content[start..].find('"') {
            let email = &content[start..start + end];
            if email.contains('@') && email.contains('.') {
                return Some(email.to_string());
            }
        }
    }
    
    // 备用：尝试匹配 "email":"xxx@xxx.com" 格式
    if let Some(pos) = content.find("\"email\":\"") {
        let start = pos + 9;
        if let Some(end) = content[start..].find('"') {
            let email = &content[start..start + end];
            if email.contains('@') && email.contains('.') {
                return Some(email.to_string());
            }
        }
    }
    
    None
}

#[derive(Serialize)]
pub struct ListUsersResult {
    pub users: Vec<UserProfile>,
    #[serde(rename = "currentUser")]
    pub current_user: Option<String>,
}

/// 获取用户列表
#[tauri::command]
pub fn list_users() -> ListUsersResult {
    let config = load_config();
    ListUsersResult {
        users: config.users,
        current_user: config.current_user,
    }
}

/// 切换用户
#[tauri::command]
pub fn switch_user(email: String) -> Result<(), String> {
    let mut config = load_config();
    
    // 检查用户是否存在
    let user_exists = config.users.iter().any(|u| u.email == email);
    if !user_exists {
        return Err(format!("用户 {} 不存在", email));
    }
    
    // 停止 RoxyBrowser
    super::process::stop_roxy()?;
    
    // 保存当前用户数据
    if let Some(ref current) = config.current_user {
        backup_roxy_data(current)?;
    }
    
    // 恢复目标用户数据
    restore_roxy_data(&email)?;
    
    // 更新配置
    config.current_user = Some(email.clone());
    for user in &mut config.users {
        if user.email == email {
            user.last_used = Utc::now().to_rfc3339();
        }
    }
    save_config(&config)?;
    
    // 启动 RoxyBrowser
    super::process::start_roxy()?;
    
    Ok(())
}

/// 删除用户
#[tauri::command]
pub fn delete_user(email: String) -> Result<(), String> {
    let mut config = load_config();
    
    // 不能删除当前用户
    if config.current_user.as_ref() == Some(&email) {
        return Err("无法删除当前活动用户".to_string());
    }
    
    // 从配置中移除
    config.users.retain(|u| u.email != email);
    save_config(&config)?;
    
    // 删除备份目录
    let profile_dir = get_profiles_dir().join(&email);
    if profile_dir.exists() {
        fs::remove_dir_all(&profile_dir).ok();
    }
    
    Ok(())
}

/// 更新用户备注
#[tauri::command]
pub fn update_user_note(email: String, note: String) -> Result<(), String> {
    let mut config = load_config();
    
    // 查找并更新用户备注
    let user_found = config.users.iter_mut().find(|u| u.email == email);
    if let Some(user) = user_found {
        user.note = note;
        save_config(&config)?;
        Ok(())
    } else {
        Err(format!("用户 {} 不存在", email))
    }
}

/// 准备添加新用户（保存当前用户并清空登录状态）
#[tauri::command]
pub fn prepare_for_new_user() -> Result<(), String> {
    let config = load_config();
    
    // 停止 RoxyBrowser
    super::process::stop_roxy()?;
    
    // 保存当前用户数据
    if let Some(ref current) = config.current_user {
        backup_roxy_data(current)?;
    }
    
    // 清空 RoxyBrowser 登录数据
    let roxy_dir = get_roxy_data_dir();
    for item in BACKUP_ITEMS {
        let path = roxy_dir.join(item);
        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(&path).ok();
            } else {
                fs::remove_file(&path).ok();
            }
        }
    }
    
    // 启动 RoxyBrowser 供用户登录
    super::process::start_roxy()?;
    
    Ok(())
}

/// 完成新用户添加（读取新用户信息并保存）
#[tauri::command]
pub fn finalize_new_user() -> Result<UserProfile, String> {
    // 停止 RoxyBrowser
    super::process::stop_roxy()?;
    
    // 从 RoxyBrowser 数据中读取新用户邮箱
    let email = read_current_email_from_roxy()
        .ok_or("无法检测到登录用户，请确保已完成登录")?;
    
    // 保存新用户数据
    backup_roxy_data(&email)?;
    
    // 更新配置
    let mut config = load_config();
    let now = Utc::now().to_rfc3339();
    
    // 检查用户是否已存在
    if !config.users.iter().any(|u| u.email == email) {
        config.users.push(UserProfile {
            email: email.clone(),
            display_name: email.split('@').next().unwrap_or(&email).to_string(),
            created_at: now.clone(),
            last_used: now.clone(),
            note: String::new(),
        });
    }
    
    config.current_user = Some(email.clone());
    save_config(&config)?;
    
    // 返回新用户信息
    let user = config.users.iter().find(|u| u.email == email).cloned()
        .ok_or("无法找到新用户配置")?;
    
    // 启动 RoxyBrowser
    super::process::start_roxy()?;
    
    Ok(user)
}

/// 导出所有用户配置到指定目录
#[tauri::command]
pub fn export_profiles(export_path: String) -> Result<String, String> {
    let profiles_dir = get_profiles_dir();
    let config = load_config();
    
    if !profiles_dir.exists() || config.users.is_empty() {
        return Err("没有可导出的用户配置".to_string());
    }
    
    let export_dir = PathBuf::from(&export_path);
    fs::create_dir_all(&export_dir)
        .map_err(|e| format!("无法创建导出目录: {}", e))?;
    
    // 导出配置文件
    let config_content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(export_dir.join("config.json"), config_content)
        .map_err(|e| format!("写入配置失败: {}", e))?;
    
    // 导出每个用户的配置目录
    let export_profiles_dir = export_dir.join("profiles");
    fs::create_dir_all(&export_profiles_dir)
        .map_err(|e| format!("无法创建profiles目录: {}", e))?;
    
    for user in &config.users {
        let src = profiles_dir.join(&user.email);
        let dst = export_profiles_dir.join(&user.email);
        if src.exists() {
            copy_dir_all(&src, &dst)?;
        }
    }
    
    Ok(format!("成功导出 {} 个用户配置到 {}", config.users.len(), export_path))
}

/// 从指定目录导入用户配置
#[tauri::command]
pub fn import_profiles(import_path: String) -> Result<String, String> {
    let import_dir = PathBuf::from(&import_path);
    
    if !import_dir.exists() {
        return Err("导入目录不存在".to_string());
    }
    
    let config_path = import_dir.join("config.json");
    if !config_path.exists() {
        return Err("导入目录中未找到 config.json".to_string());
    }
    
    // 读取导入的配置
    let import_config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置失败: {}", e))?;
    let import_config: AppConfig = serde_json::from_str(&import_config_content)
        .map_err(|e| format!("解析配置失败: {}", e))?;
    
    // 合并到现有配置
    let mut current_config = load_config();
    let mut imported_count = 0;
    
    for import_user in import_config.users {
        // 检查用户是否已存在
        if !current_config.users.iter().any(|u| u.email == import_user.email) {
            // 复制用户配置目录
            let src = import_dir.join("profiles").join(&import_user.email);
            let dst = get_profiles_dir().join(&import_user.email);
            if src.exists() {
                copy_dir_all(&src, &dst)?;
            }
            current_config.users.push(import_user);
            imported_count += 1;
        }
    }
    
    save_config(&current_config)?;
    
    Ok(format!("成功导入 {} 个用户配置", imported_count))
}
