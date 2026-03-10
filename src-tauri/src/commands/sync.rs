use chrono::Utc;
use reqwest::{Client, Method, StatusCode, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::tempdir;
use walkdir::WalkDir;
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use super::{
    profile,
    settings::{
        get_webdav_settings, load_settings, normalize_webdav_settings, update_sync_status,
        WebDavSettings, WebDavSettingsInput,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebDavSnapshotMeta {
    pub id: String,
    pub created_at: String,
    pub user_count: usize,
    pub current_user: Option<String>,
    #[serde(default)]
    pub user_emails: Vec<String>,
    pub source_platform: String,
    pub source_host: String,
    pub size_bytes: u64,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncOperationResult {
    pub message: String,
    pub snapshot_id: Option<String>,
    pub synced_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct SnapshotIndex {
    snapshots: Vec<WebDavSnapshotMeta>,
}

fn current_host_name() -> String {
    ["COMPUTERNAME", "HOSTNAME"]
        .iter()
        .find_map(|key| std::env::var(key).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn snapshot_id_now() -> String {
    Utc::now().format("%Y%m%dT%H%M%S%3fZ").to_string()
}

fn remote_segments(settings: &WebDavSettings) -> Vec<String> {
    settings
        .remote_dir
        .split('/')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn build_url(base_url: &str, segments: &[String]) -> Result<Url, String> {
    let mut url = Url::parse(base_url).map_err(|e| format!("WebDAV 地址无效: {}", e))?;
    {
        let mut path_segments = url
            .path_segments_mut()
            .map_err(|_| "WebDAV 地址格式不支持路径拼接".to_string())?;
        for segment in segments {
            path_segments.push(segment);
        }
    }
    Ok(url)
}

fn auth(builder: reqwest::RequestBuilder, settings: &WebDavSettings) -> reqwest::RequestBuilder {
    builder.basic_auth(settings.username.clone(), Some(settings.password.clone()))
}

async fn response_error(prefix: &str, response: reqwest::Response) -> String {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if body.trim().is_empty() {
        format!("{} (HTTP {})", prefix, status.as_u16())
    } else {
        format!(
            "{} (HTTP {}): {}",
            prefix,
            status.as_u16(),
            body.trim().chars().take(200).collect::<String>()
        )
    }
}

async fn ensure_collection(
    client: &Client,
    settings: &WebDavSettings,
    segments: &[String],
) -> Result<(), String> {
    let url = build_url(&settings.base_url, segments)?;
    let method = Method::from_bytes(b"MKCOL").map_err(|e| format!("无法构造 MKCOL 请求: {}", e))?;
    let response = auth(client.request(method, url), settings)
        .send()
        .await
        .map_err(|e| format!("无法连接 WebDAV 服务: {}", e))?;

    match response.status() {
        StatusCode::CREATED
        | StatusCode::METHOD_NOT_ALLOWED
        | StatusCode::OK
        | StatusCode::NO_CONTENT => Ok(()),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err("WebDAV 认证失败，请检查用户名或密码".to_string()),
        StatusCode::CONFLICT => Err("WebDAV 目录创建失败，请检查服务器基础路径是否正确".to_string()),
        _ => Err(response_error("无法创建远端目录", response).await),
    }
}

async fn ensure_remote_structure(client: &Client, settings: &WebDavSettings) -> Result<(), String> {
    let root_segments = remote_segments(settings);
    if root_segments.is_empty() {
        return Err("远端同步目录不能为空".to_string());
    }

    let mut current_segments = Vec::new();
    for segment in &root_segments {
        current_segments.push(segment.clone());
        ensure_collection(client, settings, &current_segments).await?;
    }

    let mut snapshots_segments = root_segments;
    snapshots_segments.push("snapshots".to_string());
    ensure_collection(client, settings, &snapshots_segments).await
}

async fn get_optional_json<T: DeserializeOwned>(
    client: &Client,
    settings: &WebDavSettings,
    segments: &[String],
) -> Result<Option<T>, String> {
    let url = build_url(&settings.base_url, segments)?;
    let response = auth(client.get(url), settings)
        .send()
        .await
        .map_err(|e| format!("WebDAV 请求失败: {}", e))?;

    match response.status() {
        StatusCode::NOT_FOUND => Ok(None),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err("WebDAV 认证失败，请检查用户名或密码".to_string()),
        status if status.is_success() => {
            let bytes = response
                .bytes()
                .await
                .map_err(|e| format!("读取 WebDAV 响应失败: {}", e))?;
            let data = serde_json::from_slice::<T>(&bytes)
                .map_err(|e| format!("解析远端数据失败: {}", e))?;
            Ok(Some(data))
        }
        _ => Err(response_error("读取远端数据失败", response).await),
    }
}

async fn put_bytes(
    client: &Client,
    settings: &WebDavSettings,
    segments: &[String],
    body: Vec<u8>,
    content_type: &str,
) -> Result<(), String> {
    let url = build_url(&settings.base_url, segments)?;
    let response = auth(client.put(url), settings)
        .header("Content-Type", content_type)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("上传 WebDAV 文件失败: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(response_error("上传 WebDAV 文件失败", response).await)
    }
}

async fn put_json<T: Serialize>(
    client: &Client,
    settings: &WebDavSettings,
    segments: &[String],
    value: &T,
) -> Result<(), String> {
    let body = serde_json::to_vec_pretty(value)
        .map_err(|e| format!("序列化同步元数据失败: {}", e))?;
    put_bytes(client, settings, segments, body, "application/json").await
}

async fn delete_remote_file(
    client: &Client,
    settings: &WebDavSettings,
    segments: &[String],
    allow_missing: bool,
) -> Result<(), String> {
    let url = build_url(&settings.base_url, segments)?;
    let response = auth(client.request(Method::DELETE, url), settings)
        .send()
        .await
        .map_err(|e| format!("删除 WebDAV 文件失败: {}", e))?;

    match response.status() {
        StatusCode::NOT_FOUND if allow_missing => Ok(()),
        status if status.is_success() => Ok(()),
        _ => Err(response_error("删除 WebDAV 文件失败", response).await),
    }
}

async fn download_bytes(
    client: &Client,
    settings: &WebDavSettings,
    segments: &[String],
) -> Result<Vec<u8>, String> {
    let url = build_url(&settings.base_url, segments)?;
    let response = auth(client.get(url), settings)
        .send()
        .await
        .map_err(|e| format!("下载 WebDAV 文件失败: {}", e))?;

    match response.status() {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err("WebDAV 认证失败，请检查用户名或密码".to_string()),
        status if status.is_success() => response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|e| format!("读取 WebDAV 下载内容失败: {}", e)),
        _ => Err(response_error("下载 WebDAV 文件失败", response).await),
    }
}

fn add_directory_to_archive(
    writer: &mut ZipWriter<File>,
    source_dir: &Path,
    root_name: &str,
) -> Result<(), String> {
    if !source_dir.exists() {
        return Ok(());
    }

    let file_options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    let dir_options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);

    writer
        .add_directory(format!("{}/", root_name), dir_options)
        .map_err(|e| format!("写入压缩目录失败: {}", e))?;

    for entry in WalkDir::new(source_dir) {
        let entry = entry.map_err(|e| format!("遍历目录失败: {}", e))?;
        let path = entry.path();
        if path == source_dir {
            continue;
        }

        let relative = path
            .strip_prefix(source_dir)
            .map_err(|e| format!("处理压缩路径失败: {}", e))?;
        let name = format!("{}/{}", root_name, relative.to_string_lossy().replace('\\', "/"));

        if entry.file_type().is_dir() {
            writer
                .add_directory(format!("{}/", name), dir_options)
                .map_err(|e| format!("写入压缩目录失败: {}", e))?;
            continue;
        }

        writer
            .start_file(name, file_options)
            .map_err(|e| format!("创建压缩文件失败: {}", e))?;
        let mut source_file = File::open(path).map_err(|e| format!("打开待压缩文件失败: {}", e))?;
        std::io::copy(&mut source_file, writer).map_err(|e| format!("写入压缩文件失败: {}", e))?;
    }

    Ok(())
}

fn create_sync_archive(output_path: &Path) -> Result<(crate::models::user::AppConfig, u64), String> {
    let config = profile::load_config();
    let archive_file = File::create(output_path).map_err(|e| format!("创建同步压缩包失败: {}", e))?;
    let mut writer = ZipWriter::new(archive_file);
    let file_options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let config_content = serde_json::to_vec_pretty(&config)
        .map_err(|e| format!("序列化本地配置失败: {}", e))?;
    writer
        .start_file("config.json", file_options)
        .map_err(|e| format!("写入配置文件到压缩包失败: {}", e))?;
    writer
        .write_all(&config_content)
        .map_err(|e| format!("写入压缩包失败: {}", e))?;

    let profiles_dir = profile::get_profiles_dir();
    add_directory_to_archive(&mut writer, &profiles_dir, "profiles")?;

    writer
        .finish()
        .map_err(|e| format!("完成压缩包写入失败: {}", e))?;

    let size_bytes = fs::metadata(output_path)
        .map_err(|e| format!("读取压缩包大小失败: {}", e))?
        .len();

    Ok((config, size_bytes))
}

fn extract_archive(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let archive_file = File::open(archive_path).map_err(|e| format!("打开压缩包失败: {}", e))?;
    let mut archive = ZipArchive::new(archive_file).map_err(|e| format!("解析压缩包失败: {}", e))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("读取压缩包条目失败: {}", e))?;
        let Some(relative_path) = entry.enclosed_name().map(|path| path.to_owned()) else {
            continue;
        };
        let output_path = destination.join(relative_path);

        if entry.is_dir() {
            fs::create_dir_all(&output_path)
                .map_err(|e| format!("创建解压目录失败: {}", e))?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建解压目录失败: {}", e))?;
        }

        let mut output_file = File::create(&output_path).map_err(|e| format!("创建解压文件失败: {}", e))?;
        std::io::copy(&mut entry, &mut output_file)
            .map_err(|e| format!("写入解压文件失败: {}", e))?;
    }

    Ok(())
}

fn create_local_restore_point(label: &str) -> Result<PathBuf, String> {
    let restore_root = profile::get_app_config_dir().join("restore_points");
    fs::create_dir_all(&restore_root).map_err(|e| format!("创建本地恢复点目录失败: {}", e))?;

    let safe_label = label
        .chars()
        .map(|char| if char.is_ascii_alphanumeric() || char == '-' || char == '_' { char } else { '_' })
        .collect::<String>();
    let restore_dir = restore_root.join(format!(
        "{}_{}",
        Utc::now().format("%Y%m%dT%H%M%S"),
        safe_label
    ));
    fs::create_dir_all(&restore_dir).map_err(|e| format!("创建本地恢复点失败: {}", e))?;

    let config_path = profile::get_config_path();
    if config_path.exists() {
        fs::copy(&config_path, restore_dir.join("config.json"))
            .map_err(|e| format!("写入本地恢复点配置失败: {}", e))?;
    }

    let profiles_dir = profile::get_profiles_dir();
    if profiles_dir.exists() {
        profile::copy_dir_all(&profiles_dir, &restore_dir.join("profiles"))?;
    }

    Ok(restore_dir)
}

fn replace_local_data(extracted_dir: &Path) -> Result<(), String> {
    let extracted_config = extracted_dir.join("config.json");
    if !extracted_config.exists() {
        return Err("同步包缺少 config.json".to_string());
    }

    let config_content = fs::read_to_string(&extracted_config)
        .map_err(|e| format!("读取同步包配置失败: {}", e))?;
    let _: crate::models::user::AppConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("同步包中的 config.json 无法解析: {}", e))?;

    let app_config_dir = profile::get_app_config_dir();
    fs::create_dir_all(&app_config_dir).map_err(|e| format!("创建应用配置目录失败: {}", e))?;

    let config_path = profile::get_config_path();
    if config_path.exists() {
        fs::remove_file(&config_path).map_err(|e| format!("清理本地配置失败: {}", e))?;
    }

    let profiles_dir = profile::get_profiles_dir();
    if profiles_dir.exists() {
        fs::remove_dir_all(&profiles_dir).map_err(|e| format!("清理本地 profiles 失败: {}", e))?;
    }

    fs::copy(extracted_config, &config_path).map_err(|e| format!("恢复本地配置失败: {}", e))?;

    let extracted_profiles = extracted_dir.join("profiles");
    if extracted_profiles.exists() {
        profile::copy_dir_all(&extracted_profiles, &profiles_dir)?;
    }

    Ok(())
}

fn apply_snapshot_bytes(archive_bytes: Vec<u8>, label: &str) -> Result<(), String> {
    let temp_dir = tempdir().map_err(|e| format!("创建临时目录失败: {}", e))?;
    let archive_path = temp_dir.path().join("snapshot.zip");
    fs::write(&archive_path, archive_bytes).map_err(|e| format!("写入临时同步包失败: {}", e))?;

    let extract_dir = temp_dir.path().join("extracted");
    fs::create_dir_all(&extract_dir).map_err(|e| format!("创建解压目录失败: {}", e))?;
    extract_archive(&archive_path, &extract_dir)?;

    create_local_restore_point(label)?;
    replace_local_data(&extract_dir)
}

fn snapshot_file_segments(settings: &WebDavSettings, file_name: &str) -> Vec<String> {
    let mut segments = remote_segments(settings);
    segments.push("snapshots".to_string());
    segments.push(file_name.to_string());
    segments
}

fn latest_segments(settings: &WebDavSettings) -> Vec<String> {
    let mut segments = remote_segments(settings);
    segments.push("latest.json".to_string());
    segments
}

fn index_segments(settings: &WebDavSettings) -> Vec<String> {
    let mut segments = remote_segments(settings);
    segments.push("index.json".to_string());
    segments
}

fn http_client() -> Result<Client, String> {
    Client::builder()
        .build()
        .map_err(|e| format!("创建 WebDAV 客户端失败: {}", e))
}

fn validate_connection_settings(settings: &WebDavSettings, require_enabled: bool) -> Result<(), String> {
    if require_enabled && !settings.enabled {
        return Err("请先在设置中启用 WebDAV 同步".to_string());
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

async fn load_validated_settings(require_enabled: bool) -> Result<WebDavSettings, String> {
    let settings = get_webdav_settings()?;
    validate_connection_settings(&settings, require_enabled)?;
    Ok(settings)
}

async fn sync_to_webdav_inner() -> Result<SyncOperationResult, String> {
    let settings = load_validated_settings(true).await?;
    let client = http_client()?;
    ensure_remote_structure(&client, &settings).await?;

    let flushed_user = profile::flush_current_user_backup()?;
    let temp_dir = tempdir().map_err(|e| format!("创建临时目录失败: {}", e))?;
    let snapshot_id = snapshot_id_now();
    let archive_name = format!("{}.zip", snapshot_id);
    let archive_path = temp_dir.path().join(&archive_name);
    let (config, size_bytes) = create_sync_archive(&archive_path)?;
    let archive_bytes = fs::read(&archive_path).map_err(|e| format!("读取同步压缩包失败: {}", e))?;

    let metadata = WebDavSnapshotMeta {
        id: snapshot_id.clone(),
        created_at: Utc::now().to_rfc3339(),
        user_count: config.users.len(),
        current_user: config.current_user.clone(),
        user_emails: config.users.iter().map(|user| user.email.clone()).collect(),
        source_platform: std::env::consts::OS.to_string(),
        source_host: current_host_name(),
        size_bytes,
        file_name: archive_name.clone(),
    };

    put_bytes(
        &client,
        &settings,
        &snapshot_file_segments(&settings, &archive_name),
        archive_bytes,
        "application/zip",
    )
    .await?;
    put_json(
        &client,
        &settings,
        &snapshot_file_segments(&settings, &format!("{}.json", snapshot_id)),
        &metadata,
    )
    .await?;

    let mut index = get_optional_json::<SnapshotIndex>(&client, &settings, &index_segments(&settings))
        .await?
        .unwrap_or_default();
    index.snapshots.retain(|snapshot| snapshot.id != metadata.id);
    index.snapshots.insert(0, metadata.clone());

    put_json(&client, &settings, &index_segments(&settings), &index).await?;
    put_json(&client, &settings, &latest_segments(&settings), &metadata).await?;

    let message = if let Some(user) = flushed_user {
        format!("已同步到 WebDAV，并保存当前活动用户 {} 的最新本地快照", user)
    } else {
        "已同步到 WebDAV".to_string()
    };

    Ok(SyncOperationResult {
        message,
        snapshot_id: Some(metadata.id),
        synced_at: Some(metadata.created_at),
    })
}

async fn list_snapshots_inner() -> Result<Vec<WebDavSnapshotMeta>, String> {
    let settings = load_validated_settings(true).await?;
    let client = http_client()?;
    let index = get_optional_json::<SnapshotIndex>(&client, &settings, &index_segments(&settings))
        .await?
        .unwrap_or_default();
    Ok(index.snapshots)
}

async fn restore_snapshot_by_meta(metadata: WebDavSnapshotMeta) -> Result<SyncOperationResult, String> {
    let settings = load_validated_settings(true).await?;
    let client = http_client()?;
    let archive_bytes = download_bytes(
        &client,
        &settings,
        &snapshot_file_segments(&settings, &metadata.file_name),
    )
    .await?;

    apply_snapshot_bytes(archive_bytes, &metadata.id)?;

    Ok(SyncOperationResult {
        message: format!("已恢复远端快照 {}", metadata.id),
        snapshot_id: Some(metadata.id),
        synced_at: Some(Utc::now().to_rfc3339()),
    })
}

async fn pull_latest_inner() -> Result<SyncOperationResult, String> {
    let settings = load_validated_settings(true).await?;
    let client = http_client()?;
    let latest = get_optional_json::<WebDavSnapshotMeta>(&client, &settings, &latest_segments(&settings))
        .await?
        .ok_or_else(|| "远端还没有可下载的同步快照".to_string())?;
    restore_snapshot_by_meta(latest).await
}

async fn restore_snapshot_inner(snapshot_id: String) -> Result<SyncOperationResult, String> {
    let settings = load_validated_settings(true).await?;
    let client = http_client()?;
    let metadata_segments = snapshot_file_segments(&settings, &format!("{}.json", snapshot_id));
    let metadata = get_optional_json::<WebDavSnapshotMeta>(&client, &settings, &metadata_segments)
        .await?
        .ok_or_else(|| format!("未找到远端历史快照 {}", snapshot_id))?;
    restore_snapshot_by_meta(metadata).await
}

async fn delete_snapshot_inner(snapshot_id: String) -> Result<SyncOperationResult, String> {
    let settings = load_validated_settings(true).await?;
    let client = http_client()?;

    let mut index = get_optional_json::<SnapshotIndex>(&client, &settings, &index_segments(&settings))
        .await?
        .unwrap_or_default();

    let snapshot = index
        .snapshots
        .iter()
        .find(|entry| entry.id == snapshot_id)
        .cloned()
        .ok_or_else(|| format!("未找到远端备份 {}", snapshot_id))?;

    delete_remote_file(
        &client,
        &settings,
        &snapshot_file_segments(&settings, &snapshot.file_name),
        true,
    )
    .await?;
    delete_remote_file(
        &client,
        &settings,
        &snapshot_file_segments(&settings, &format!("{}.json", snapshot.id)),
        true,
    )
    .await?;

    index.snapshots.retain(|entry| entry.id != snapshot_id);
    put_json(&client, &settings, &index_segments(&settings), &index).await?;

    if let Some(latest) = index.snapshots.first() {
        put_json(&client, &settings, &latest_segments(&settings), latest).await?;
    } else {
        delete_remote_file(&client, &settings, &latest_segments(&settings), true).await?;
    }

    Ok(SyncOperationResult {
        message: format!("已删除远端备份 {}", snapshot_id),
        snapshot_id: index.snapshots.first().map(|entry| entry.id.clone()),
        synced_at: Some(Utc::now().to_rfc3339()),
    })
}

#[tauri::command]
pub async fn test_webdav_connection(settings: Option<WebDavSettingsInput>) -> Result<String, String> {
    let resolved_settings = if let Some(input) = settings {
        let app_settings = load_settings().unwrap_or_default();
        normalize_webdav_settings(input, &app_settings.webdav)
    } else {
        get_webdav_settings()?
    };
    validate_connection_settings(&resolved_settings, false)?;

    let client = http_client()?;
    ensure_remote_structure(&client, &resolved_settings).await?;
    Ok("WebDAV 连接成功，远端目录可访问".to_string())
}

#[tauri::command]
pub async fn sync_to_webdav() -> Result<SyncOperationResult, String> {
    let attempt_time = Utc::now().to_rfc3339();
    match sync_to_webdav_inner().await {
        Ok(result) => {
            update_sync_status(
                "success",
                result.message.clone(),
                result.snapshot_id.clone(),
                result
                    .synced_at
                    .clone()
                    .unwrap_or_else(|| attempt_time.clone()),
            )?;
            Ok(result)
        }
        Err(error) => {
            let _ = update_sync_status("error", error.clone(), None, attempt_time);
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn list_webdav_snapshots() -> Result<Vec<WebDavSnapshotMeta>, String> {
    list_snapshots_inner().await
}

#[tauri::command]
pub async fn pull_latest_from_webdav() -> Result<SyncOperationResult, String> {
    let attempt_time = Utc::now().to_rfc3339();
    match pull_latest_inner().await {
        Ok(result) => {
            update_sync_status(
                "success",
                result.message.clone(),
                result.snapshot_id.clone(),
                result
                    .synced_at
                    .clone()
                    .unwrap_or_else(|| attempt_time.clone()),
            )?;
            Ok(result)
        }
        Err(error) => {
            let _ = update_sync_status("error", error.clone(), None, attempt_time);
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn restore_webdav_snapshot(snapshot_id: String) -> Result<SyncOperationResult, String> {
    let attempt_time = Utc::now().to_rfc3339();
    match restore_snapshot_inner(snapshot_id).await {
        Ok(result) => {
            update_sync_status(
                "success",
                result.message.clone(),
                result.snapshot_id.clone(),
                result
                    .synced_at
                    .clone()
                    .unwrap_or_else(|| attempt_time.clone()),
            )?;
            Ok(result)
        }
        Err(error) => {
            let _ = update_sync_status("error", error.clone(), None, attempt_time);
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn delete_webdav_snapshot(snapshot_id: String) -> Result<SyncOperationResult, String> {
    delete_snapshot_inner(snapshot_id).await
}
