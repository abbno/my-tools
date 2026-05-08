use actix_web::{web, HttpResponse};
use actix_files::NamedFile;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::fs::File;
use std::io::Write;
use crate::models::{App, Version, ChangelogItem, CreateAppRequest, VersionInfoResponse};
use crate::storage::Storage;
use chrono::Utc;

/// Format file size to human readable format
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{} KB", bytes / 1024)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{} MB", bytes / (1024 * 1024))
    } else {
        format!("{} GB", bytes / (1024 * 1024 * 1024))
    }
}

/// Parse plain text changelog into ChangelogItem list
/// Format: each line is a description
fn parse_changelog_text(text: &str) -> Vec<ChangelogItem> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| ChangelogItem {
            item_type: "update".to_string(),
            description: line.trim().to_string(),
        })
        .collect()
}

pub struct AppState {
    pub storage: Storage,
}

// API: Get latest version for client
pub async fn get_latest_version(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let app_id = path.into_inner();

    match state.storage.get_latest_version(&app_id) {
        Ok(Some(version)) => {
            let response: VersionInfoResponse = version.into();
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().body("App or version not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// API: Get specific version
pub async fn get_version(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (app_id, version) = path.into_inner();

    match state.storage.get_version(&app_id, &version) {
        Ok(Some(v)) => {
            let response: VersionInfoResponse = v.into();
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().body("Version not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// API: Download file
pub async fn download_file(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> actix_web::Result<NamedFile> {
    let (app_id, filename) = path.into_inner();
    let file_path = state.storage.get_files_dir(&app_id).join(&filename);
    NamedFile::open(file_path).map_err(|e| e.into())
}

// API: Get all apps (JSON)
pub async fn get_apps(state: web::Data<AppState>) -> HttpResponse {
    match state.storage.load_apps() {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// API: Create new app
pub async fn create_app(
    state: web::Data<AppState>,
    body: web::Json<CreateAppRequest>,
) -> HttpResponse {
    let req = body.into_inner();
    let app = App {
        id: req.id,
        name: req.name,
        created_at: Utc::now().to_rfc3339(),
    };

    match state.storage.add_app(app) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// API: Get app versions
pub async fn get_app_versions(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let app_id = path.into_inner();

    match state.storage.load_versions(&app_id) {
        Ok(versions) => HttpResponse::Ok().json(versions),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// API: Upload new version with file
pub async fn upload_version(
    state: web::Data<AppState>,
    path: web::Path<String>,
    mut payload: Multipart,
) -> actix_web::Result<HttpResponse> {
    let app_id = path.into_inner();

    // Collect form data
    let mut version_num: Option<String> = None;
    let mut changelog: Vec<ChangelogItem> = vec![];
    let mut file_name: Option<String> = None;
    let mut file_size: u64 = 0;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let name = field.name().to_string();

        if name == "version" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk?);
            }
            version_num = Some(String::from_utf8_lossy(&data).to_string());
        } else if name == "changelog" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk?);
            }
            let log_str = String::from_utf8_lossy(&data).to_string();
            // Parse plain text changelog: each line is "type description"
            changelog = parse_changelog_text(&log_str);
        } else if name == "file" {
            // Save file
            let filename = field.content_disposition().get_filename().unwrap_or("unknown").to_string();
            file_name = Some(filename.clone());
            let files_dir = state.storage.get_files_dir(&app_id);
            let file_path = files_dir.join(&filename);
            let mut f = File::create(&file_path)?;
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                file_size += data.len() as u64;
                f.write_all(&data)?;
            }
        }
    }

    if let (Some(version_num), Some(file_name)) = (version_num, file_name) {
        let version = Version {
            app_id: app_id.clone(),
            version: version_num,
            release_date: Utc::now().format("%Y-%m-%d").to_string(),
            changelog,
            download_url: format!("/download/{}/{}", app_id, file_name),
            file_name,
            file_size,
            created_at: Utc::now().to_rfc3339(),
        };

        state.storage.add_version(version)?;
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().body("Missing version or file"))
    }
}

// API: Delete version
pub async fn delete_version(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (app_id, version) = path.into_inner();

    match state.storage.delete_version(&app_id, &version) {
        Ok(true) => HttpResponse::Ok().finish(),
        Ok(false) => HttpResponse::NotFound().body("Version not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// Page: Index (Home)
pub async fn index_page(state: web::Data<AppState>) -> HttpResponse {
    let apps = match state.storage.load_apps() {
        Ok(a) => a,
        Err(_) => vec![],
    };

    let apps_html = apps.iter().map(|app| {
        format!(
            r#"<div class="card">
    <div class="card-header">
        <div class="card-title">{}</div>
    </div>
    <div class="card-meta">
        <div class="card-meta-item">
            <span class="card-meta-label">标识：</span>
            <span class="card-meta-value">{}</span>
        </div>
        <div class="card-meta-item">
            <span class="card-meta-label">创建时间：</span>
            <span class="card-meta-value">{}</span>
        </div>
    </div>
    <div class="card-footer">
        <a href="/app/{}" class="btn btn-sm">详情 →</a>
    </div>
</div>"#,
            app.name, app.id, app.created_at, app.id
        )
    }).collect::<Vec<_>>().join("\n");

    let empty_msg = if apps.is_empty() {
        r#"<div class="card text-center"><p style="color: var(--text-secondary);">暂无软件，请创建一个。</p></div>"#
    } else {
        ""
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>版本管理服务</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div class="container">
        <header class="header">
            <h1 class="header-title">版本管理服务</h1>
            <a href="/app/new" class="btn btn-primary">+ 新建软件</a>
        </header>
        <h2 style="color: var(--text-secondary); margin-bottom: 24px;">软件列表</h2>
        {}
        {}
    </div>
</body>
</html>"#,
        apps_html, empty_msg
    );

    HttpResponse::Ok().content_type("text/html").body(html)
}

// Page: App Detail
pub async fn app_detail_page(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let app_id = path.into_inner();

    let app = match state.storage.get_app(&app_id) {
        Ok(Some(a)) => a,
        _ => return HttpResponse::NotFound().body("软件不存在"),
    };

    let versions = match state.storage.load_versions(&app_id) {
        Ok(v) => v,
        Err(_) => vec![],
    };

    let versions_html = versions.iter().map(|v| {
        let changelog_html = v.changelog.iter().map(|c| {
            format!(
                r#"<div class="changelog-item">
    <span class="changelog-icon">●</span>
    <span>{}</span>
</div>"#,
                c.description
            )
        }).collect::<Vec<_>>().join("\n");

        format!(
            r#"<div class="version-item">
    <div class="version-header">
        <span class="version-number">v{}</span>
        <span class="version-date">{}</span>
    </div>
    <div class="changelog-list">
        {}
    </div>
    <div class="file-info">
        文件：{} ({})
    </div>
    <div class="card-footer">
        <a href="{}" class="btn btn-sm">下载</a>
        <button class="btn btn-sm btn-danger" onclick="deleteVersion('{}')">删除</button>
    </div>
</div>"#,
            v.version, v.release_date, changelog_html, v.file_name, format_size(v.file_size), v.download_url, v.version
        )
    }).collect::<Vec<_>>().join("\n");

    let empty_msg = if versions.is_empty() {
        r#"<div class="card text-center"><p style="color: var(--text-secondary);">暂无版本，请上传一个。</p></div>"#
    } else {
        ""
    };

    // Build HTML without format! to avoid escaping braces in JS
    let html = String::new()
        + r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>"# + &app.name + r#"</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div class="container">
        <div style="margin-bottom: 24px;">
            <a href="/" class="btn btn-sm">← 返回首页</a>
        </div>
        <header class="header">
            <h1 class="header-title">"# + &app.name + r#"</h1>
        </header>
        <div style="margin-bottom: 32px;">
            <button class="btn btn-primary" onclick="showUploadModal()">⬆ 上传新版本</button>
        </div>
        <h2 style="color: var(--text-secondary); margin-bottom: 24px;">版本历史</h2>
"# + &versions_html + "\n" + empty_msg + r#"
        <!-- Upload Modal -->
        <div id="uploadModal" class="modal-overlay">
            <div class="modal-content">
                <h3 class="modal-title">上传新版本</h3>
                <form id="uploadForm" action="/api/apps/"# + &app_id + r#"/versions/form" method="post" enctype="multipart/form-data">
                    <div class="form-group">
                        <label class="form-label">版本号</label>
                        <input type="text" name="version" class="form-input" placeholder="0.1.0" required>
                    </div>
                    <div class="form-group">
                        <label class="form-label">安装包文件</label>
                        <input type="file" name="file" class="form-input" required>
                    </div>
                    <div class="form-group">
                        <label class="form-label">更新日志</label>
                        <textarea name="changelog" class="form-input" rows="4" placeholder="新增在线升级功能&#10;修复连接状态异常&#10;优化启动速度"></textarea>
                        <p style="color: var(--text-muted); font-size: 12px; margin-top: 4px;">每行一条更新内容</p>
                    </div>
                    <div class="flex flex-between" style="margin-top: 24px;">
                        <button type="button" class="btn" onclick="hideUploadModal()">取消</button>
                        <button type="submit" class="btn btn-primary">上传 →</button>
                    </div>
                </form>
            </div>
        </div>
        <script>
            function showUploadModal() {
                document.getElementById('uploadModal').style.display = 'block';
            }
            function hideUploadModal() {
                document.getElementById('uploadModal').style.display = 'none';
            }
            function deleteVersion(version) {
                if (confirm('确定删除版本 ' + version + ' 吗？')) {
                    fetch('/api/apps/"# + &app_id + r#"/versions/' + version, { method: 'DELETE' })
                        .then(r => { if (r.ok) location.reload(); });
                }
            }
        </script>
    </div>
</body>
</html>"#;

    HttpResponse::Ok().content_type("text/html").body(html)
}

// Page: New App
pub async fn new_app_page() -> HttpResponse {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>新建软件</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div class="container">
        <div style="margin-bottom: 24px;">
            <a href="/" class="btn btn-sm">← 返回首页</a>
        </div>
        <header class="header">
            <h1 class="header-title">新建软件</h1>
        </header>
        <div class="card">
            <form action="/api/apps/form" method="post">
                <div class="form-group">
                    <label class="form-label">软件标识（唯一标识符）</label>
                    <input type="text" name="id" class="form-input" placeholder="ssh-tunnel-manager" required>
                </div>
                <div class="form-group">
                    <label class="form-label">软件名称（显示名称）</label>
                    <input type="text" name="name" class="form-input" placeholder="SSH隧道管理器" required>
                </div>
                <div class="flex flex-between" style="margin-top: 24px;">
                    <a href="/" class="btn">取消</a>
                    <button type="submit" class="btn btn-primary">创建 →</button>
                </div>
            </form>
        </div>
    </div>
</body>
</html>"#.to_string();

    HttpResponse::Ok().content_type("text/html").body(html)
}

// Handle form submission for create_app (HTML form)
pub async fn create_app_form(
    state: web::Data<AppState>,
    form: web::Form<CreateAppRequest>,
) -> HttpResponse {
    let req = form.into_inner();
    let app = App {
        id: req.id,
        name: req.name,
        created_at: Utc::now().to_rfc3339(),
    };

    match state.storage.add_app(app) {
        Ok(_) => HttpResponse::Found()
            .insert_header(("Location", "/"))
            .finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// Handle form submission for upload_version (HTML form)
pub async fn upload_version_form(
    state: web::Data<AppState>,
    path: web::Path<String>,
    mut payload: Multipart,
) -> actix_web::Result<HttpResponse> {
    let app_id = path.into_inner();

    // Collect form data
    let mut version_num: Option<String> = None;
    let mut changelog: Vec<ChangelogItem> = vec![];
    let mut file_name: Option<String> = None;
    let mut file_size: u64 = 0;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let name = field.name().to_string();

        if name == "version" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk?);
            }
            version_num = Some(String::from_utf8_lossy(&data).to_string());
        } else if name == "changelog" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk?);
            }
            let log_str = String::from_utf8_lossy(&data).to_string();
            // Parse plain text changelog: each line is "type description"
            changelog = parse_changelog_text(&log_str);
        } else if name == "file" {
            let filename = field.content_disposition().get_filename().unwrap_or("unknown").to_string();
            file_name = Some(filename.clone());
            let files_dir = state.storage.get_files_dir(&app_id);
            let file_path = files_dir.join(&filename);
            let mut f = File::create(&file_path)?;
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                file_size += data.len() as u64;
                f.write_all(&data)?;
            }
        }
    }

    if let (Some(version_num), Some(file_name)) = (version_num, file_name) {
        let version = Version {
            app_id: app_id.clone(),
            version: version_num,
            release_date: Utc::now().format("%Y-%m-%d").to_string(),
            changelog,
            download_url: format!("/download/{}/{}", app_id, file_name),
            file_name,
            file_size,
            created_at: Utc::now().to_rfc3339(),
        };

        state.storage.add_version(version)?;
    }

    // Redirect back to app detail page after upload
    Ok(HttpResponse::Found()
        .insert_header(("Location", format!("/app/{}", app_id)))
        .finish())
}