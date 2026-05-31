// SSH 密钥设置模块
// 用于自动生成和部署 SSH 密钥

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ssh_key::{PrivateKey, Algorithm};

use crate::utils::logger;

/// 密钥设置请求（不再需要密码，用户在 CMD 窗口中输入）
#[derive(Debug, Deserialize)]
pub struct SetupKeyRequest {
    pub host: String,
    pub port: i32,
    pub username: String,
}

/// 密钥设置结果
#[derive(Debug, Serialize)]
pub struct SetupKeyResult {
    pub success: bool,
    pub key_path: Option<String>,
    pub message: String,
}

/// 获取 SSH 目录路径
pub fn get_ssh_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir()
        .ok_or_else(|| "无法获取用户主目录".to_string())?;
    let ssh_dir = home.join(".ssh");

    // 确保 .ssh 目录存在
    if !ssh_dir.exists() {
        fs::create_dir_all(&ssh_dir)
            .map_err(|e| format!("创建 .ssh 目录失败: {}", e))?;
        logger::info(&format!("创建 SSH 目录: {}", ssh_dir.display()));
    }

    Ok(ssh_dir)
}

/// 获取默认密钥路径
pub fn get_default_key_path() -> Result<PathBuf, String> {
    let ssh_dir = get_ssh_dir()?;
    Ok(ssh_dir.join("id_ed25519"))
}

/// 检查本地是否已有 SSH 密钥
pub fn check_existing_key() -> Option<PathBuf> {
    match get_default_key_path() {
        Ok(path) if path.exists() => {
            logger::info(&format!("发现已有 SSH 密钥: {}", path.display()));
            Some(path)
        }
        _ => None,
    }
}

/// 生成 Ed25519 密钥
pub fn generate_ed25519_key(key_path: &PathBuf) -> Result<String, String> {
    logger::info("开始生成 Ed25519 密钥...");

    // 生成 Ed25519 私钥
    let mut rng = ssh_key::rand_core::OsRng;
    let private_key = PrivateKey::random(&mut rng, Algorithm::Ed25519)
        .map_err(|e| format!("密钥生成失败: {}", e))?;

    // 获取公钥
    let public_key = private_key.public_key();

    // 编码为 OpenSSH 格式
    let private_openssh = private_key
        .to_openssh(ssh_key::LineEnding::LF)
        .map_err(|e| format!("私钥编码失败: {}", e))?;

    let public_openssh = public_key
        .to_openssh()
        .map_err(|e| format!("公钥编码失败: {}", e))?;

    // 保存私钥
    let mut private_file = fs::File::create(key_path)
        .map_err(|e| format!("创建私钥文件失败: {}", e))?;
    private_file.write_all(private_openssh.as_bytes())
        .map_err(|e| format!("写入私钥失败: {}", e))?;

    // 设置私钥权限（仅对 Unix 有效）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(key_path, fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("设置私钥权限失败: {}", e))?;
    }

    // 保存公钥
    let public_path = key_path.with_extension("pub");
    let mut public_file = fs::File::create(&public_path)
        .map_err(|e| format!("创建公钥文件失败: {}", e))?;
    public_file.write_all(public_openssh.as_bytes())
        .map_err(|e| format!("写入公钥失败: {}", e))?;

    logger::info(&format!("密钥生成成功: {}", key_path.display()));

    Ok(public_openssh.to_string())
}

/// 使用 PowerShell 窗口部署公钥（Windows）- 用户交互输入密码
#[cfg(target_os = "windows")]
pub fn deploy_public_key_cmd(
    host: &str,
    port: i32,
    username: &str,
    public_key: &str,
) -> Result<(), String> {
    use std::process::Command;

    logger::info(&format!("弹出 PowerShell 窗口部署公钥到 {}@{}:{} ...", username, host, port));

    let temp_dir = std::env::temp_dir();
    let temp_ps1 = temp_dir.join("deploy_ssh_key.ps1");
    let ps1_path_str = temp_ps1.to_string_lossy().to_string();

    // 公钥嵌入本地 PS 脚本，远程命令用 @'...'@ 单引号 here-string 构建
    let pubkey = public_key.trim().replace('\'', "''");
    let ps1_content = format!(
        "$ErrorActionPreference = 'Stop'\r\n\
# ---- 远程命令（@'...'@ = PS 单引号 here-string）----\r\n\
$remote = @'\r\n\
# 1. 写入用户 authorized_keys（始终成功）\r\n\
$ukDir = \"$env:USERPROFILE\\.ssh\"\r\n\
$uk = \"$ukDir\\authorized_keys\"\r\n\
if(!(Test-Path $ukDir)){{ New-Item -ItemType Directory $ukDir -Force | Out-Null }}\r\n\
Add-Content $uk '{pubkey}'\r\n\
Write-Host \"[OK] Key written to user authorized_keys\"\r\n\
# 2. 尝试写入管理员 authorized_keys（可能被权限拒绝，正常）\r\n\
$akDir = \"$env:ProgramData\\ssh\"\r\n\
if(Test-Path $akDir){{\r\n\
    try {{\r\n\
        Add-Content \"$akDir\\administrators_authorized_keys\" '{pubkey}' -ErrorAction Stop\r\n\
        icacls \"$akDir\\administrators_authorized_keys\" /inheritance:r /grant \"SYSTEM:(R)\" /grant \"BUILTIN\\Administrators:(R)\" 2>$null\r\n\
        Write-Host \"[OK] Admin authorized_keys written + ACL fixed\"\r\n\
    }} catch {{\r\n\
        Write-Host \"[INFO] Cannot write admin authorized_keys (expected for non-elevated SSH)\"\r\n\
        Write-Host \"[INFO] Key is already in user profile's authorized_keys\"\r\n\
    }}\r\n\
}} else {{\r\n\
    Write-Host \"[OK] Linux/compat server - user authorized_keys is sufficient\"\r\n\
}}\r\n\
exit 0\r\n\
'@\r\n\
$encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($remote))\r\n\
Write-Host '========================================'\r\n\
Write-Host 'Deploying SSH key to {username}@{host}:{port}'\r\n\
Write-Host '========================================'\r\n\
Write-Host ''\r\n\
Write-Host 'Please enter your server password when prompted:'\r\n\
Write-Host ''\r\n\
ssh -o StrictHostKeyChecking=accept-new -p {port} {username}@{host} \"powershell -EncodedCommand $encoded\"\r\n\
Write-Host ''\r\n\
Write-Host 'Deployment complete. Press any key to close...' -ForegroundColor Yellow\r\n\
$null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')\r\n\
Remove-Item -Force '{ps1_path}' -ErrorAction SilentlyContinue\r\n",
        pubkey = pubkey,
        username = username,
        host = host,
        port = port,
        ps1_path = ps1_path_str.replace('\'', "''"),
    );

    fs::write(&temp_ps1, ps1_content.as_bytes())
        .map_err(|e| format!("写入脚本文件失败: {}", e))?;

    logger::info(&format!("PowerShell 脚本路径: {}", ps1_path_str));

    // Start-Process 弹出新窗口，-Wait 等待用户关闭
    let status = Command::new("powershell")
        .args([
            "-NoProfile", "-ExecutionPolicy", "Bypass",
            "-Command",
            &format!("Start-Process powershell -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File \"{}\"' -Wait", ps1_path_str),
        ])
        .status()
        .map_err(|e| format!("启动 PowerShell 失败: {}", e))?;

    let _ = fs::remove_file(&temp_ps1);

    logger::info(&format!("PowerShell 窗口已关闭 (status: {})", status));
    Ok(())
}

/// 验证密钥是否已成功部署到服务器，返回 (success, error_detail)
#[cfg(target_os = "windows")]
pub fn verify_key_deployment(
    host: &str,
    port: i32,
    username: &str,
    key_path: &PathBuf,
) -> Result<(bool, String), String> {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    logger::info(&format!("验证密钥部署: {}@{}:{} using key {}", username, host, port, key_path.display()));

    let output = Command::new("ssh")
        .args(["-i", &key_path.to_string_lossy()])
        .args(["-o", "BatchMode=yes"])
        .args(["-o", "StrictHostKeyChecking=accept-new"])
        .args(["-o", "ConnectTimeout=10"])
        .args(["-p", &port.to_string()])
        .arg(format!("{}@{}", username, host))
        .arg("exit")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("验证命令执行失败: {}", e))?;

    if output.status.success() {
        logger::info("密钥验证成功");
        Ok((true, String::new()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = parse_ssh_error_for_verify(&stderr);
        logger::info(&format!("密钥验证失败: {} → {}", stderr.trim(), detail));
        Ok((false, detail))
    }
}

/// 解析验证失败时的 SSH 错误信息
fn parse_ssh_error_for_verify(stderr: &str) -> String {
    let s = stderr.trim();
    if s.is_empty() {
        return "SSH 服务器拒绝了密钥认证：请确认密钥已正确部署到服务器".to_string();
    }
    if s.contains("Permission denied") || s.contains("publickey") {
        return "密钥认证被拒绝：密钥未在服务器上注册，或部署到了错误的位置".to_string();
    }
    if s.contains("Host key verification failed") {
        return "主机密钥验证失败".to_string();
    }
    if s.contains("Connection timed out") || s.contains("connect to host") {
        return format!("无法连接到服务器: {}", s.lines().next().unwrap_or(s));
    }
    s.lines().next().unwrap_or("未知验证错误").to_string()
}

/// Linux/Mac 实现（使用 sshpass 或类似方法）
#[cfg(not(target_os = "windows"))]
pub fn deploy_public_key_cmd(
    host: &str,
    port: i32,
    username: &str,
    public_key: &str,
) -> Result<(), String> {
    // Linux/Mac 可以使用终端交互方式
    use std::process::Command;

    logger::info(&format!("弹出终端部署公钥到 {}@{}:{} ...", username, host, port));

    let ssh_cmd = format!(
        "ssh -o StrictHostKeyChecking=accept-new -p {} {}@{} \"mkdir -p ~/.ssh; chmod 700 ~/.ssh; echo '{}' >> ~/.ssh/authorized_keys; chmod 600 ~/.ssh/authorized_keys\"",
        port, username, host, public_key
    );

    // 使用 x-terminal-emulator 或 gnome-terminal 等
    let status = Command::new("x-terminal-emulator")
        .args(["-e", &ssh_cmd])
        .status()
        .map_err(|e| format!("启动终端失败: {} (请确保有终端模拟器)", e))?;

    logger::info(&format!("终端已关闭 (status: {})", status));
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn verify_key_deployment(
    host: &str,
    port: i32,
    username: &str,
    key_path: &PathBuf,
) -> Result<(bool, String), String> {
    use std::process::Command;

    logger::info(&format!("验证密钥部署: {}@{}:{} using key {}", username, host, port, key_path.display()));

    let output = Command::new("ssh")
        .args(["-i", &key_path.to_string_lossy()])
        .args(["-o", "BatchMode=yes"])
        .args(["-o", "StrictHostKeyChecking=accept-new"])
        .args(["-o", "ConnectTimeout=10"])
        .args(["-p", &port.to_string()])
        .arg(format!("{}@{}", username, host))
        .arg("exit")
        .output()
        .map_err(|e| format!("验证命令执行失败: {}", e))?;

    if output.status.success() {
        logger::info("密钥验证成功");
        Ok((true, String::new()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        logger::info(&format!("密钥验证失败: {}", stderr));
        Ok((false, stderr.trim().to_string()))
    }
}

/// 完整的密钥设置流程
pub async fn setup_ssh_key(request: SetupKeyRequest) -> Result<SetupKeyResult, String> {
    logger::info(&format!(
        "开始自动密钥设置: {}@{}:{}",
        request.username, request.host, request.port
    ));

    // 1. 检查现有密钥或生成新密钥
    let key_path = match check_existing_key() {
        Some(path) => {
            logger::info("使用已有密钥");
            path
        }
        None => {
            let path = get_default_key_path()?;
            generate_ed25519_key(&path)?;
            path
        }
    };

    // 读取公钥
    let public_path = key_path.with_extension("pub");
    let public_key = fs::read_to_string(&public_path)
        .map_err(|e| format!("读取公钥失败: {}", e))?;

    // 2. 弹出 CMD 窗口让用户输入密码部署公钥
    deploy_public_key_cmd(
        &request.host,
        request.port,
        &request.username,
        &public_key
    )?;

    // 3. 验证密钥是否部署成功
    let (verified, verify_detail) = verify_key_deployment(
        &request.host,
        request.port,
        &request.username,
        &key_path
    )?;

    if verified {
        logger::info(&format!("密钥设置完成: {}", key_path.display()));
        Ok(SetupKeyResult {
            success: true,
            key_path: Some(key_path.to_string_lossy().to_string()),
            message: "密钥设置成功，配置已自动更新为密钥认证".to_string(),
        })
    } else {
        // 验证失败，给出针对性提示
        #[allow(unused_parens)]
        let detail_info = (if verify_detail.contains("publickey") || verify_detail.contains("Permission denied") {
            let pk = public_key.trim();
            format!(
                "管理员账户的 SSH 免密登录需要额外配置。\n\
                 方案一（推荐）：在服务器上创建普通用户（非管理员组），使用普通用户连接。\n\
                 方案二：远程桌面登录服务器，用管理员身份运行以下命令：\n\
                 ----------------------------------------\n\
                 Add-Content \"$env:ProgramData\\ssh\\administrators_authorized_keys\" \"{pk}\"\n\
                 icacls \"$env:ProgramData\\ssh\\administrators_authorized_keys\" /inheritance:r /grant \"SYSTEM:(R)\" /grant \"BUILTIN\\Administrators:(R)\"\n\
                 ----------------------------------------\n\
                 或者注释掉 sshd_config 中 Match Group Administrators 相关的行，然后运行 Restart-Service sshd",
                pk = pk
            )
        } else {
            format!("密钥验证失败: {}", verify_detail)
        });

        logger::info(&format!("密钥设置验证失败: {}", verify_detail));
        Ok(SetupKeyResult {
            success: false,
            key_path: None,
            message: detail_info,
        })
    }
}