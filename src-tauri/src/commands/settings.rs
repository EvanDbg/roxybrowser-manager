use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(rename = "roxyExePath")]
    pub roxy_exe_path: Option<String>,
    #[serde(
        rename = "autoDetectEnabled",
        alias = "auto_detect_enabled",
        default = "default_auto_detect"
    )]
    pub auto_detect_enabled: bool,
    #[serde(default)]
    pub webdav: WebDavSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebDavSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_remote_dir")]
    pub remote_dir: String,
    #[serde(default = "default_auto_sync")]
    pub auto_sync_enabled: bool,
    #[serde(default)]
    pub last_sync_at: Option<String>,
    #[serde(default)]
    pub last_sync_status: Option<String>,
    #[serde(default)]
    pub last_sync_message: Option<String>,
    #[serde(default)]
    pub last_snapshot_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebDavSettingsInput {
    pub enabled: bool,
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub remote_dir: String,
    pub auto_sync_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub enabled: bool,
    pub auto_sync_enabled: bool,
    pub last_sync_at: Option<String>,
    pub last_sync_status: Option<String>,
    pub last_sync_message: Option<String>,
    pub last_snapshot_id: Option<String>,
}

fn default_auto_detect() -> bool {
    true
}

fn default_auto_sync() -> bool {
    true
}

fn default_remote_dir() -> String {
    "roxybrowser-manager".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            roxy_exe_path: None,
            auto_detect_enabled: true,
            webdav: WebDavSettings::default(),
        }
    }
}

impl Default for WebDavSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: String::new(),
            username: String::new(),
            password: String::new(),
            remote_dir: default_remote_dir(),
            auto_sync_enabled: true,
            last_sync_at: None,
            last_sync_status: None,
            last_sync_message: None,
            last_snapshot_id: None,
        }
    }
}

impl From<&WebDavSettings> for SyncStatus {
    fn from(value: &WebDavSettings) -> Self {
        Self {
            enabled: value.enabled,
            auto_sync_enabled: value.auto_sync_enabled,
            last_sync_at: value.last_sync_at.clone(),
            last_sync_status: value.last_sync_status.clone(),
            last_sync_message: value.last_sync_message.clone(),
            last_snapshot_id: value.last_snapshot_id.clone(),
        }
    }
}

fn get_settings_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir().ok_or_else(|| "无法获取配置目录".to_string())?;
    let app_config_dir = config_dir.join("roxybrowser-manager");

    if !app_config_dir.exists() {
        std::fs::create_dir_all(&app_config_dir)
            .map_err(|e| format!("无法创建配置目录: {}", e))?;
    }

    Ok(app_config_dir.join("settings.json"))
}

pub fn load_settings() -> Result<AppSettings, String> {
    let settings_path = get_settings_path()?;

    if !settings_path.exists() {
        return Ok(AppSettings::default());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("无法读取配置文件: {}", e))?;

    let settings: AppSettings = serde_json::from_str(&content)
        .map_err(|e| format!("配置文件格式错误: {}", e))?;

    Ok(settings)
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let settings_path = get_settings_path()?;
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("无法序列化配置: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("无法写入配置文件: {}", e))?;

    Ok(())
}

pub fn validate_path(path: &str) -> Result<bool, String> {
    let path_buf = PathBuf::from(path);

    if !path_buf.exists() {
        return Ok(false);
    }

    if !path_buf.is_file() {
        return Ok(false);
    }

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

pub fn validate_webdav_settings(settings: &WebDavSettings) -> Result<(), String> {
    if !settings.enabled {
        return Ok(());
    }

    if settings.base_url.trim().is_empty() {
        return Err("请填写 WebDAV 服务器地址".to_string());
    }

    if settings.username.trim().is_empty() {
        return Err("请填写 WebDAV 用户名".to_string());
    }

    if settings.password.trim().is_empty() {
        return Err("请填写 WebDAV 密码".to_string());
    }

    if settings.remote_dir.trim().is_empty() {
        return Err("请填写远端同步目录".to_string());
    }

    Ok(())
}

pub fn normalize_webdav_settings(input: WebDavSettingsInput, previous: &WebDavSettings) -> WebDavSettings {
    let remote_dir = input
        .remote_dir
        .trim()
        .trim_matches('/')
        .to_string();

    WebDavSettings {
        enabled: input.enabled,
        base_url: input.base_url.trim().trim_end_matches('/').to_string(),
        username: input.username.trim().to_string(),
        password: input.password,
        remote_dir: if remote_dir.is_empty() {
            default_remote_dir()
        } else {
            remote_dir
        },
        auto_sync_enabled: input.auto_sync_enabled,
        last_sync_at: previous.last_sync_at.clone(),
        last_sync_status: previous.last_sync_status.clone(),
        last_sync_message: previous.last_sync_message.clone(),
        last_snapshot_id: previous.last_snapshot_id.clone(),
    }
}

pub fn update_sync_status(
    status: &str,
    message: String,
    snapshot_id: Option<String>,
    sync_time: String,
) -> Result<(), String> {
    let mut settings = load_settings().unwrap_or_default();
    settings.webdav.last_sync_status = Some(status.to_string());
    settings.webdav.last_sync_message = Some(message);
    settings.webdav.last_sync_at = Some(sync_time);
    if let Some(snapshot_id) = snapshot_id {
        settings.webdav.last_snapshot_id = Some(snapshot_id);
    }
    save_settings(&settings)
}

pub fn get_enhanced_default_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Some(local_app_data) = dirs::data_local_dir() {
            paths.push(
                local_app_data
                    .join("Programs")
                    .join("RoxyBrowser")
                    .join("RoxyBrowser.exe"),
            );
        }

        paths.push(PathBuf::from(r"C:\Program Files\RoxyBrowser\RoxyBrowser.exe"));
        paths.push(PathBuf::from(r"C:\Program Files (x86)\RoxyBrowser\RoxyBrowser.exe"));

        for drive in &['D', 'E', 'F'] {
            paths.push(PathBuf::from(format!(
                r"{}:\Program Files\RoxyBrowser\RoxyBrowser.exe",
                drive
            )));
            paths.push(PathBuf::from(format!(
                r"{}:\Program Files (x86)\RoxyBrowser\RoxyBrowser.exe",
                drive
            )));
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

#[tauri::command]
pub fn get_roxy_exe_path() -> Result<Option<String>, String> {
    let settings = load_settings()?;
    Ok(settings.roxy_exe_path)
}

#[tauri::command]
pub fn set_roxy_exe_path(path: String) -> Result<(), String> {
    if !validate_path(&path)? {
        return Err("无效的 RoxyBrowser 可执行文件路径".to_string());
    }

    let mut settings = load_settings().unwrap_or_default();
    settings.roxy_exe_path = Some(path);
    save_settings(&settings)?;
    Ok(())
}

#[tauri::command]
pub fn validate_roxy_exe_path(path: String) -> Result<bool, String> {
    validate_path(&path)
}

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

#[tauri::command]
pub fn get_webdav_settings() -> Result<WebDavSettings, String> {
    let settings = load_settings()?;
    Ok(settings.webdav)
}

#[tauri::command]
pub fn save_webdav_settings(settings: WebDavSettingsInput) -> Result<WebDavSettings, String> {
    let mut app_settings = load_settings().unwrap_or_default();
    let normalized = normalize_webdav_settings(settings, &app_settings.webdav);
    validate_webdav_settings(&normalized)?;
    app_settings.webdav = normalized.clone();
    save_settings(&app_settings)?;
    Ok(normalized)
}

#[tauri::command]
pub fn get_sync_status() -> Result<SyncStatus, String> {
    let settings = load_settings()?;
    Ok(SyncStatus::from(&settings.webdav))
}

#[tauri::command]
pub async fn browse_for_exe(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let file_path = app
        .dialog()
        .file()
        .set_title("选择 RoxyBrowser 可执行文件")
        .add_filter("可执行文件", &["exe"])
        .blocking_pick_file();

    if let Some(path) = file_path {
        if let Some(path_ref) = path.as_path() {
            let path_str = path_ref.to_string_lossy().to_string();
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

#[tauri::command]
pub async fn browse_for_folder(app: tauri::AppHandle, title: String) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let folder_path = app
        .dialog()
        .file()
        .set_title(&title)
        .blocking_pick_folder();

    if let Some(path) = folder_path {
        if let Some(path_ref) = path.as_path() {
            Ok(Some(path_ref.to_string_lossy().to_string()))
        } else {
            Err("无法获取文件夹路径".to_string())
        }
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn clear_roxy_exe_path() -> Result<(), String> {
    let mut settings = load_settings().unwrap_or_default();
    settings.roxy_exe_path = None;
    save_settings(&settings)?;
    Ok(())
}
