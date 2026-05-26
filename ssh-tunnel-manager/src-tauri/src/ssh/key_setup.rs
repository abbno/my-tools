// SSH 密钥设置模块
// 用于自动生成和部署 SSH 密钥

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ssh_key::{PrivateKey, Algorithm};

use crate::utils::logger;

/// 密钥设置请求
#[derive(Debug, Deserialize)]
pub struct SetupKeyRequest {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
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

/// 使用 SSH 命令部署公钥（Windows 使用 PowerShell 方式）
#[cfg(target_os = "windows")]
pub fn deploy_public_key(
    host: &str,
    port: i32,
    username: &str,
    password: &str,
    public_key: &str,
) -> Result<(), String> {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    logger::info(&format!("开始部署公钥到 {}@{}:{} ...", username, host, port));

    // 创建临时 PowerShell 脚本文件来处理密码传递
    // 因为 SSH 不接受命令行密码输入

    let temp_script = std::env::temp_dir().join("deploy_ssh_key.ps1");
    // PowerShell here-string 格式
    let script_content = format!(
r#"
$password = '{password}'
$pubkey = '{pubkey}'
$host = '{host}'
$port = '{port}'
$user = '{user}'

$cmd = "mkdir -p ~/.ssh; chmod 700 ~/.ssh; echo `$pubkey >> ~/.ssh/authorized_keys; chmod 600 ~/.ssh/authorized_keys"
echo $password | ssh -o StrictHostKeyChecking=accept-new -o PreferredAuthentications=password -o PubkeyAuthentication=no -p $port $user@$host $cmd
"#,
        password = password,
        pubkey = public_key,
        host = host,
        port = port,
        user = username
    );

    fs::write(&temp_script, &script_content)
        .map_err(|e| format!("写入临时脚本失败: {}", e))?;

    let output = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-File", &temp_script.to_string_lossy()])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行 PowerShell 失败: {}", e))?;

    // 删除临时脚本
    fs::remove_file(&temp_script).ok();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("部署公钥失败: {}", stderr));
    }

    logger::info("公钥部署成功");
    Ok(())
}

/// 使用 sshpass 部署公钥（Linux/Mac）
#[cfg(not(target_os = "windows"))]
pub fn deploy_public_key(
    host: &str,
    port: i32,
    username: &str,
    password: &str,
    public_key: &str,
) -> Result<(), String> {
    use std::process::Command;

    logger::info(&format!("开始部署公钥到 {}@{}:{} ...", username, host, port));

    // 使用 sshpass 传递密码
    let ssh_target = format!("{}@{}:{}", username, host, port);
    let cmd = format!(
        "mkdir -p ~/.ssh && chmod 700 ~/.ssh && echo '{}' >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys",
        public_key
    );

    let output = Command::new("sshpass")
        .args(["-p", password])
        .args(["ssh", "-o", "StrictHostKeyChecking=accept-new"])
        .args(["-o", "PreferredAuthentications=password"])
        .args(["-o", "PubkeyAuthentication=no"])
        .args(["-p", &port.to_string()])
        .arg(&ssh_target)
        .arg(&cmd)
        .output()
        .map_err(|e| format!("执行 sshpass 失败: {} (请确保已安装 sshpass)", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("部署公钥失败: {}", stderr));
    }

    logger::info("公钥部署成功");
    Ok(())
}

/// 完整的密钥设置流程
pub async fn setup_ssh_key(request: SetupKeyRequest) -> Result<SetupKeyResult, String> {
    logger::info(&format!(
        "开始自动密钥设置: {}@{}:{}",
        request.username, request.host, request.port
    ));

    // 1. 检查现有密钥
    let key_path = match check_existing_key() {
        Some(path) => {
            logger::info("使用已有密钥");
            path
        }
        None => {
            // 2. 生成新密钥
            let path = get_default_key_path()?;
            generate_ed25519_key(&path)?;
            path
        }
    };

    // 读取公钥
    let public_path = key_path.with_extension("pub");
    let public_key = fs::read_to_string(&public_path)
        .map_err(|e| format!("读取公钥失败: {}", e))?;

    // 3. 部署公钥到服务器（同步操作）
    deploy_public_key(
        &request.host,
        request.port,
        &request.username,
        &request.password,
        &public_key
    )?;

    logger::info(&format!("密钥设置完成: {}", key_path.display()));

    Ok(SetupKeyResult {
        success: true,
        key_path: Some(key_path.to_string_lossy().to_string()),
        message: "密钥设置成功，配置已自动更新为密钥认证".to_string(),
    })
}