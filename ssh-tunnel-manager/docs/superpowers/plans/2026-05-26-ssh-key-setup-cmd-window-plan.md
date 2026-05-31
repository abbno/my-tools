# 自动 SSH 密钥设置功能实现计划 - CMD 窗口交互方案

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 CMD 窗口交互式 SSH 密钥部署，用户在弹出窗口中输入密码，完成后自动检测验证

**Architecture:** 修改后端 deploy_public_key 函数使用 cmd 窗口交互，添加 verify_key_deployment 函数进行自动检测，前端调整对话框显示状态和重试逻辑

**Tech Stack:** Rust (Tauri), Vue 3 (TDesign), PowerShell/CMD

---

## Task 1: 后端 - 修改公钥部署函数

**Files:**
- Modify: `src-tauri/src/ssh/key_setup.rs`

- [ ] **Step 1: 删除旧的 PowerShell SendKeys 实现，重写为 CMD 窗口交互方式**

```rust
/// 使用 CMD 窗口部署公钥（Windows）
#[cfg(target_os = "windows")]
pub fn deploy_public_key_cmd(
    host: &str,
    port: i32,
    username: &str,
    public_key: &str,
) -> Result<(), String> {
    use std::process::Command;
    
    logger::info(&format!("弹出 CMD 窗口部署公钥到 {}@{}:{} ...", username, host, port));
    
    // 构建 SSH 命令
    let ssh_cmd = format!(
        "ssh -o StrictHostKeyChecking=accept-new -p {} {}@{} \"mkdir -p ~/.ssh; chmod 700 ~/.ssh; echo '{}' >> ~/.ssh/authorized_keys; chmod 600 ~/.ssh/authorized_keys\"",
        port, username, host, public_key
    );
    
    // 打开 CMD 窗口并等待用户操作完成
    // 使用 start /wait 让进程等待窗口关闭
    let output = Command::new("cmd")
        .args(["/C", "start", "/wait", "cmd", "/K", &ssh_cmd])
        .output()
        .map_err(|e| format!("启动 CMD 窗口失败: {}", e))?;
    
    // cmd 窗口关闭后继续
    logger::info("CMD 窗口已关闭，准备验证部署结果");
    Ok(())
}
```

- [ ] **Step 2: 添加密钥部署验证函数**

```rust
/// 验证密钥是否已成功部署到服务器
pub fn verify_key_deployment(
    host: &str,
    port: i32,
    username: &str,
    key_path: &PathBuf,
) -> Result<bool, String> {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    
    logger::info(&format!("验证密钥部署: {}@{}:{}", username, host, port));
    
    // 使用 BatchMode 尝试密钥登录
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
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        logger::info(&format!("密钥验证失败: {}", stderr));
        Ok(false)
    }
}
```

- [ ] **Step 3: 修改 setup_ssh_key 函数整合新流程**

```rust
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
    let verified = verify_key_deployment(
        &request.host,
        request.port,
        &request.username,
        &key_path
    )?;

    if verified {
        Ok(SetupKeyResult {
            success: true,
            key_path: Some(key_path.to_string_lossy().to_string()),
            message: "密钥设置成功，配置已自动更新为密钥认证".to_string(),
        })
    } else {
        Ok(SetupKeyResult {
            success: false,
            key_path: None,
            message: "密钥验证失败，可能是密码输入错误，请重试".to_string(),
        })
    }
}
```

- [ ] **Step 4: 运行 cargo check 验证编译**

---

## Task 2: 前端 - 调整密钥设置对话框

**Files:**
- Modify: `src/components/KeySetupDialog.vue`

- [ ] **Step 1: 添加重试按钮和处理逻辑**

修改对话框显示：
- 进度状态：显示"请在弹出的命令窗口中输入密码部署密钥"
- 成功状态：显示成功信息和密钥路径，"完成"按钮
- 失败状态：显示错误信息，"重试"按钮和"取消"按钮

- [ ] **Step 2: 修改 handleComplete 处理成功和失败两种情况**

---

## Task 3: 功能验证

- [ ] **Step 1: 启动开发环境测试**
- [ ] **Step 2: 创建新配置，验证 CMD 窗口弹出**
- [ ] **Step 3: 输入正确密码，验证密钥部署成功**
- [ ] **Step 4: 输入错误密码，验证重试功能**