use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use std::time::Duration;

/// 应用名称（用于构建更新 URL）
const APP_NAME: &str = "ssh-tunnel-manager";

/// 更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// 新版本号
    pub version: String,
    /// 发布日期
    pub release_date: String,
    /// 更新日志
    pub changelog: Vec<ChangelogItem>,
    /// 下载地址
    pub download_url: String,
    /// 是否强制更新
    pub force_update: bool,
}

/// 更新日志项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogItem {
    /// 类型：feature/fix/improve
    #[serde(rename = "type")]
    pub item_type: String,
    /// 描述
    pub description: String,
}

/// 下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// 已下载字节
    pub downloaded: u64,
    /// 总字节
    pub total: u64,
    /// 百分比
    pub percentage: u8,
}

/// 服务器版本信息响应结构
#[derive(Debug, Clone, Deserialize)]
struct ServerVersionInfo {
    /// 版本号
    version: String,
    /// 发布日期
    #[serde(default)]
    release_date: Option<String>,
    /// 下载地址
    #[serde(default)]
    download_url: Option<String>,
    /// 是否强制更新
    #[serde(default)]
    force_update: Option<bool>,
    /// 更新日志
    #[serde(default)]
    changelog: Option<Vec<ChangelogItem>>,
}

/// 构建更新检查 URL
fn build_update_url(server_url: &str) -> String {
    format!("{}/api/version/{}", server_url.trim_end_matches('/'), APP_NAME)
}

/// 检查更新
/// 使用自定义 HTTP 请求检查更新服务器
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    // 从数据库读取更新服务器地址
    let server_url = crate::db::get_app_setting("update_server_url")
        .map_err(|e| format!("读取更新服务器配置失败: {}", e))?;

    match server_url {
        None => Err("请先配置更新服务器地址".to_string()),
        Some(url) => {
            let url = url.trim();
            if url.is_empty() {
                return Err("请先配置更新服务器地址".to_string());
            }

            // 构建更新检查 URL
            let check_url = build_update_url(url);

            // 创建 HTTP 客户端
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

            // 发送请求
            let response = client
                .get(&check_url)
                .send()
                .await
                .map_err(|e| format!("请求更新服务器失败: {}", e))?;

            // 检查响应状态
            if !response.status().is_success() {
                return Err(format!("服务器返回错误状态: {}", response.status()));
            }

            // 解析响应
            let version_info: ServerVersionInfo = response
                .json()
                .await
                .map_err(|e| format!("解析服务器响应失败: {}", e))?;

            // 获取当前版本
            let current_version = app.package_info().version.to_string();

            // 比较版本号
            if version_info.version.is_empty() || version_info.version == current_version {
                return Ok(None); // 无更新
            }

            // 构建下载 URL
            let download_url = version_info.download_url.unwrap_or_else(|| {
                // 如果服务器没有返回下载 URL，构建默认 URL
                format!(
                    "{}/download/{}/{}.exe",
                    url.trim_end_matches('/'),
                    version_info.version,
                    APP_NAME
                )
            });

            // 返回更新信息
            Ok(Some(UpdateInfo {
                version: version_info.version,
                release_date: version_info.release_date.unwrap_or_default(),
                changelog: version_info.changelog.unwrap_or_default(),
                download_url,
                force_update: version_info.force_update.unwrap_or(false),
            }))
        }
    }
}

/// 下载并安装更新
/// 注意：调用此方法后应用会自动退出并重启
#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), String> {
    // 从数据库读取更新服务器地址
    let server_url = crate::db::get_app_setting("update_server_url")
        .map_err(|e| format!("读取更新服务器配置失败: {}", e))?
        .unwrap_or_default();

    // 首先检查更新以获取下载 URL
    let update_info = check_update(app.clone()).await?;

    match update_info {
        None => Err("没有可用的更新".to_string()),
        Some(info) => {
            // 拼接完整下载 URL
            let full_download_url = if info.download_url.starts_with("http://") || info.download_url.starts_with("https://") {
                // 已经是完整 URL，直接使用
                info.download_url.clone()
            } else {
                // 相对路径，拼接服务器地址
                format!(
                    "{}{}",
                    server_url.trim_end_matches('/'),
                    info.download_url
                )
            };

            let app_clone = app.clone();
            let download_url = full_download_url;

            // 创建 HTTP 客户端
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(300)) // 5分钟超时
                .build()
                .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

            // 发送下载请求
            let response = client
                .get(&download_url)
                .send()
                .await
                .map_err(|e| format!("下载请求失败: {}", e))?;

            if !response.status().is_success() {
                return Err(format!("下载失败，服务器返回: {}", response.status()));
            }

            // 获取内容长度
            let total_size = response.content_length().unwrap_or(0);
            let mut downloaded: u64 = 0;

            // 创建临时文件
            let temp_dir = std::env::temp_dir();
            let installer_path = temp_dir.join(format!("{}_update_{}.exe", APP_NAME, info.version));

            // 创建文件
            let mut file = std::fs::File::create(&installer_path)
                .map_err(|e| format!("创建临时文件失败: {}", e))?;

            // 下载并写入文件
            use std::io::Write;
            let mut stream = response.bytes_stream();
            use futures::StreamExt;

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| format!("读取下载数据失败: {}", e))?;
                file.write_all(&chunk)
                    .map_err(|e| format!("写入文件失败: {}", e))?;
                downloaded += chunk.len() as u64;

                // 发送进度事件
                let percentage = if total_size > 0 {
                    ((downloaded as f64 / total_size as f64) * 100.0) as u8
                } else {
                    0
                };
                let _ = app_clone.emit(
                    "update-download-progress",
                    DownloadProgress {
                        downloaded,
                        total: total_size,
                        percentage,
                    },
                );
            }

            // 发送下载完成事件
            let _ = app_clone.emit("update-download-complete", ());

            // 执行安装程序
            #[cfg(target_os = "windows")]
            {
                use std::process::Command;

                // 使用 ShellExecuteW 执行安装程序
                let installer_path_str = installer_path.to_string_lossy().to_string();

                // 退出当前应用并启动安装程序
                Command::new("cmd")
                    .args(&["/C", "start", "", &installer_path_str])
                    .spawn()
                    .map_err(|e| format!("启动安装程序失败: {}", e))?;

                // 延迟一下让安装程序启动
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // 退出应用
                app.exit(0);
            }

            #[cfg(target_os = "macos")]
            {
                // macOS: 打开 dmg 文件
                let installer_path_str = installer_path.to_string_lossy().to_string();
                std::process::Command::new("open")
                    .arg(&installer_path_str)
                    .spawn()
                    .map_err(|e| format!("打开安装包失败: {}", e))?;

                app.exit(0);
            }

            #[cfg(target_os = "linux")]
            {
                // Linux: 执行 AppImage 或其他格式
                // 设置可执行权限
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&installer_path, std::fs::Permissions::from_mode(0o755))
                        .map_err(|e| format!("设置执行权限失败: {}", e))?;
                }

                std::process::Command::new(&installer_path)
                    .spawn()
                    .map_err(|e| format!("启动安装程序失败: {}", e))?;

                app.exit(0);
            }

            Ok(())
        }
    }
}

/// 获取上次检查时间
#[tauri::command]
pub fn get_last_check_time() -> Option<String> {
    // 从持久化存储获取上次检查时间
    // 目前返回 None，后续可实现持久化
    None
}