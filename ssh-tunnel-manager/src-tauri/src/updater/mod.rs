use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use std::time::Duration;

/// 应用名称（用于构建更新 URL）
const APP_NAME: &str = "ssh-tunnel-manager";

/// 获取应用版本号
#[tauri::command]
pub fn get_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// 退出应用
#[tauri::command]
pub fn exit_app(app: AppHandle) {
    crate::utils::logger::info("用户请求退出应用");
    app.exit(0);
}

/// 更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// 新版本号
    pub version: String,
    /// 发布日期
    pub release_date: String,
    /// 更新日志
    pub changelog: Vec<ChangelogItem>,
    /// 下载地址（相对路径）
    pub download_url: String,
    /// 完整下载地址（用于前端显示）
    pub full_download_url: String,
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

/// 比较语义版本号，返回: 1 表示 a > b, -1 表示 a < b, 0 表示相等
fn compare_versions(a: &str, b: &str) -> i32 {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let a_parts = parse_version(a);
    let b_parts = parse_version(b);

    // 比较各部分
    for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
        let a_val = if i < a_parts.len() { a_parts[i] } else { 0 };
        let b_val = if i < b_parts.len() { b_parts[i] } else { 0 };
        if a_val > b_val {
            return 1;
        } else if a_val < b_val {
            return -1;
        }
    }
    0
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

            crate::utils::logger::info(&format!("检查更新，服务器地址: {}", url));

            // 构建更新检查 URL
            let check_url = build_update_url(url);

            // 创建 HTTP 客户端
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| {
                    crate::utils::logger::error(&format!("创建 HTTP 客户端失败: {}", e));
                    format!("创建 HTTP 客户端失败: {}", e)
                })?;

            // 发送请求
            let response = client
                .get(&check_url)
                .send()
                .await
                .map_err(|e| {
                    crate::utils::logger::error(&format!("请求更新服务器失败: {}", e));
                    format!("请求更新服务器失败: {}", e)
                })?;

            // 检查响应状态
            if !response.status().is_success() {
                crate::utils::logger::error(&format!("服务器返回错误状态: {}", response.status()));
                return Err(format!("服务器返回错误状态: {}", response.status()));
            }

            // 解析响应
            let version_info: ServerVersionInfo = response
                .json()
                .await
                .map_err(|e| {
                    crate::utils::logger::error(&format!("解析服务器响应失败: {}", e));
                    format!("解析服务器响应失败: {}", e)
                })?;

            // 获取当前版本
            let current_version = app.package_info().version.to_string();

            crate::utils::logger::info(&format!(
                "服务器版本: {}, 当前版本: {}",
                version_info.version, current_version
            ));

            // 比较版本号：只有服务器版本大于当前版本才需要更新
            if version_info.version.is_empty() {
                return Ok(None); // 无版本信息
            }

            // 语义版本比较：服务器版本必须大于当前版本
            if compare_versions(&version_info.version, &current_version) <= 0 {
                crate::utils::logger::info("无需更新，服务器版本不大于当前版本");
                return Ok(None); // 服务器版本不大于当前版本，无需更新
            }

            // 构建下载 URL（相对路径，与服务器返回格式一致）
            // 服务器返回格式: /download/{app_id}/{filename}
            let download_url = version_info.download_url.unwrap_or_else(|| {
                // 如果服务器没有返回下载 URL，构建默认相对路径
                format!(
                    "/download/{}/{}.exe",
                    APP_NAME,
                    APP_NAME
                )
            });

            // 构建完整下载 URL（用于前端显示）
            let full_download_url = if download_url.starts_with("http://") || download_url.starts_with("https://") {
                download_url.clone()
            } else {
                format!(
                    "{}{}",
                    url.trim_end_matches('/'),
                    download_url
                )
            };

            crate::utils::logger::info(&format!("发现新版本: {}, 下载地址: {}", version_info.version, full_download_url));

            // 返回更新信息
            Ok(Some(UpdateInfo {
                version: version_info.version,
                release_date: version_info.release_date.unwrap_or_default(),
                changelog: version_info.changelog.unwrap_or_default(),
                download_url,
                full_download_url,
                force_update: version_info.force_update.unwrap_or(false),
            }))
        }
    }
}

/// 使用管理员权限启动安装程序（Windows）
#[cfg(target_os = "windows")]
fn run_installer_as_admin(installer_path: &std::path::Path) -> Result<(), String> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::Win32::Foundation::HWND;
    use windows::core::HSTRING;

    let installer_path_str = installer_path.to_string_lossy().to_string();
    crate::utils::logger::info(&format!("使用管理员权限启动安装程序: {}", installer_path_str));

    // 使用 ShellExecuteW 的 "runas" verb 来请求管理员权限
    let operation = HSTRING::from("runas");
    let file = HSTRING::from(installer_path_str.clone());
    let parameters = HSTRING::from("");

    // ShellExecuteW 是 unsafe 函数，需要在 unsafe 块中调用
    let result = unsafe {
        ShellExecuteW(
            HWND(std::ptr::null_mut()),  // 无父窗口
            &operation,
            &file,
            &parameters,
            None,  // 默认工作目录
            SW_SHOWNORMAL,
        )
    };

    // ShellExecuteW 返回值 > 32 表示成功
    // HINSTANCE 的内部值可以通过 .0 获取
    let handle_value = result.0 as isize;
    if handle_value > 32 {
        crate::utils::logger::info("安装程序已启动（请求管理员权限）");
        Ok(())
    } else {
        let error = std::io::Error::last_os_error();
        crate::utils::logger::error(&format!("启动安装程序失败: {} (路径: {})", error, installer_path_str));
        Err(format!("启动安装程序失败: {} (路径: {})", error, installer_path_str))
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

            crate::utils::logger::info(&format!("开始下载更新，URL: {}", full_download_url));

            let app_clone = app.clone();

            // 创建 HTTP 客户端
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(300)) // 5分钟超时
                .build()
                .map_err(|e| {
                    crate::utils::logger::error(&format!("创建 HTTP 客户端失败: {}", e));
                    format!("创建 HTTP 客户端失败: {}", e)
                })?;

            // 发送下载请求
            let response = client
                .get(&full_download_url)
                .send()
                .await
                .map_err(|e| {
                    crate::utils::logger::error(&format!("下载请求失败: {}", e));
                    format!("下载请求失败: {}", e)
                })?;

            if !response.status().is_success() {
                crate::utils::logger::error(&format!("下载失败，服务器返回: {}", response.status()));
                return Err(format!("下载失败，服务器返回: {}", response.status()));
            }

            // 获取内容长度
            let total_size = response.content_length().unwrap_or(0);
            crate::utils::logger::info(&format!("下载开始: 总大小 = {} bytes", total_size));
            let mut downloaded: u64 = 0;

            // 创建临时文件
            let temp_dir = std::env::temp_dir();
            let installer_path = temp_dir.join(format!("{}_update_{}.exe", APP_NAME, info.version));
            crate::utils::logger::info(&format!("临时文件路径: {}", installer_path.display()));

            // 创建文件
            let mut file = std::fs::File::create(&installer_path)
                .map_err(|e| {
                    crate::utils::logger::error(&format!("创建临时文件失败: {}", e));
                    format!("创建临时文件失败: {}", e)
                })?;

            // 下载并写入文件
            use std::io::Write;
            let mut stream = response.bytes_stream();
            use futures::StreamExt;

            let mut last_percentage: u8 = 0;

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| {
                    crate::utils::logger::error(&format!("读取下载数据失败: {}", e));
                    format!("读取下载数据失败: {}", e)
                })?;
                file.write_all(&chunk)
                    .map_err(|e| {
                        crate::utils::logger::error(&format!("写入文件失败: {}", e));
                        format!("写入文件失败: {}", e)
                    })?;
                downloaded += chunk.len() as u64;

                // 发送进度事件
                let percentage = if total_size > 0 {
                    ((downloaded as f64 / total_size as f64) * 100.0) as u8
                } else {
                    // 如果没有总大小，根据已下载量估算进度（最大99%）
                    // 每1MB增加1%，最大99%
                    std::cmp::min(99, (downloaded / 1_048_576) as u8)
                };

                // 只在百分比变化时发送事件（减少事件频率）
                if percentage != last_percentage {
                    crate::utils::logger::debug(&format!("下载进度: {}%", percentage));
                    let _ = app_clone.emit(
                        "update-download-progress",
                        DownloadProgress {
                            downloaded,
                            total: total_size,
                            percentage,
                        },
                    );
                    last_percentage = percentage;
                }
            }

            // 确保发送100%进度
            crate::utils::logger::info(&format!("下载完成: 共 {} bytes", downloaded));
            let _ = app_clone.emit(
                "update-download-progress",
                DownloadProgress {
                    downloaded,
                    total: total_size,
                    percentage: 100,
                },
            );

            // 发送下载完成事件
            let _ = app_clone.emit("update-download-complete", ());

            // 【重要】先关闭文件句柄，再启动安装程序
            // 否则会导致 "另一个程序正在使用此文件" 错误
            file.flush().ok();
            drop(file);

            crate::utils::logger::info(&format!("准备启动安装程序: {}", installer_path.display()));

            // 执行安装程序（使用管理员权限）
            #[cfg(target_os = "windows")]
            {
                run_installer_as_admin(&installer_path)?;
            }

            #[cfg(target_os = "macos")]
            {
                let installer_path_str = installer_path.to_string_lossy().to_string();
                std::process::Command::new("open")
                    .arg(&installer_path_str)
                    .spawn()
                    .map_err(|e| format!("打开安装包失败: {}", e))?;
            }

            #[cfg(target_os = "linux")]
            {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&installer_path, std::fs::Permissions::from_mode(0o755))
                        .map_err(|e| format!("设置执行权限失败: {}", e))?;
                }

                std::process::Command::new(&installer_path)
                    .spawn()
                    .map_err(|e| format!("启动安装程序失败: {}", e))?;
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