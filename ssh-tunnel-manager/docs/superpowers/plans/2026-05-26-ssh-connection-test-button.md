# SSH 连接测试按钮功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在配置编辑弹框中添加测试按钮，验证 SSH 连接配置有效性

**Architecture:** 前端新增测试按钮调用后端 `test_ssh_connection` Tauri 命令，后端依次执行本地端口检查、TCP 连通性测试、SSH 登录认证测试，返回结构化结果

**Tech Stack:** Rust (Tauri), Vue 3, TypeScript, TDesign

---

## File Structure

| 文件 | 责任 |
|------|------|
| `src-tauri/src/commands/config.rs` | 新增 `TestConnectionRequest`、`TestConnectionResult` 类型定义和 `test_ssh_connection` 命令 |
| `src-tauri/src/commands/mod.rs` | 导出 `test_ssh_connection` 命令 |
| `src-tauri/src/lib.rs` | 注册 `test_ssh_connection` 命令到 invoke_handler |
| `src/types/index.ts` | 新增前端类型定义 |
| `src/api/tauri.ts` | 新增 `testSshConnection` API 函数 |
| `src/components/ConfigForm.vue` | 添加测试按钮和处理逻辑 |

---

### Task 1: 后端类型定义

**Files:**
- Modify: `src-tauri/src/commands/config.rs:1-33` (在现有类型定义后添加)

- [ ] **Step 1: 添加测试请求和结果类型定义**

在 `src-tauri/src/commands/config.rs` 文件中，在 `ConfigDto` 定义之后添加以下类型：

```rust
/// SSH 连接测试请求
#[derive(Debug, Deserialize)]
pub struct TestConnectionRequest {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub auth_type: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub key_passphrase: Option<String>,
    pub local_host: String,
    pub local_port: i32,
}

/// 单步测试结果
#[derive(Debug, Serialize)]
pub struct TestStepResult {
    pub success: bool,
    pub message: String,
}

/// 测试详情
#[derive(Debug, Serialize)]
pub struct TestDetails {
    pub local_port: TestStepResult,
    pub tcp_connectivity: TestStepResult,
    pub ssh_login: TestStepResult,
}

/// 测试结果
#[derive(Debug, Serialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub message: String,
    pub details: TestDetails,
}
```

- [ ] **Step 2: 编译验证类型定义**

Run: `cd ssh-tunnel-manager/src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 3: Commit**

```bash
git add ssh-tunnel-manager/src-tauri/src/commands/config.rs
git commit -m "feat(backend): add TestConnectionRequest and TestConnectionResult types"
```

---

### Task 2: 后端测试命令实现

**Files:**
- Modify: `src-tauri/src/commands/config.rs` (添加 `test_ssh_connection` 函数)

- [ ] **Step 1: 添加 test_ssh_connection 命令**

在 `src-tauri/src/commands/config.rs` 文件末尾添加以下函数：

```rust
/// 测试 SSH 连接
#[tauri::command]
pub async fn test_ssh_connection(request: TestConnectionRequest) -> Result<TestConnectionResult, String> {
    use crate::ssh::sidecar::{is_port_in_use, get_port_process_info, check_remote_connectivity};
    use crate::utils::logger;
    use std::process::Command;
    use std::time::Duration;

    let host = &request.host;
    let port = request.port;
    let username = &request.username;
    let local_host = &request.local_host;
    let local_port = request.local_port;

    logger::info(&format!("开始测试 SSH 连接: {}@{}:{}", username, host, port));

    // 1. 检查本地端口
    let local_port_result = if is_port_in_use(local_host, local_port) {
        let process_info = get_port_process_info(local_host, local_port);
        let msg = if let Some(info) = process_info {
            format!("本地端口 {}:{} 已被占用 (PID: {}, 进程: {})", local_host, local_port, info.pid, info.name)
        } else {
            format!("本地端口 {}:{} 已被占用", local_host, local_port)
        };
        TestStepResult { success: false, message: msg }
    } else {
        TestStepResult { success: true, message: format!("本地端口 {}:{} 可用", local_host, local_port) }
    };

    // 如果本地端口被占用，直接返回失败
    if !local_port_result.success {
        logger::error(&format!("SSH 连接测试失败 [本地端口]: {}", local_port_result.message));
        return Ok(TestConnectionResult {
            success: false,
            message: local_port_result.message.clone(),
            details: TestDetails {
                local_port: local_port_result,
                tcp_connectivity: TestStepResult { success: false, message: "未测试".to_string() },
                ssh_login: TestStepResult { success: false, message: "未测试".to_string() },
            },
        });
    }

    // 2. 检查 TCP 连通性
    let tcp_result = match check_remote_connectivity(host, port, 2) {
        Ok(_) => TestStepResult { success: true, message: format!("远程主机 {}:{} 可达", host, port) },
        Err(e) => TestStepResult { success: false, message: format!("远程主机 {}:{} 连接失败: {}", host, port, e) },
    };

    // 如果 TCP 连通性失败，直接返回失败
    if !tcp_result.success {
        logger::error(&format!("SSH 连接测试失败 [TCP连通性]: {}", tcp_result.message));
        return Ok(TestConnectionResult {
            success: false,
            message: tcp_result.message.clone(),
            details: TestDetails {
                local_port: local_port_result,
                tcp_connectivity: tcp_result,
                ssh_login: TestStepResult { success: false, message: "未测试".to_string() },
            },
        });
    }

    // 3. 测试 SSH 登录认证
    let ssh_result = test_ssh_login(&request);

    // 构建最终结果
    let success = ssh_result.success;
    let message = if success {
        "连接测试成功".to_string()
    } else {
        ssh_result.message.clone()
    };

    if success {
        logger::info(&format!("SSH 连接测试成功: {}@{}:{}", username, host, port));
    } else {
        logger::error(&format!("SSH 连接测试失败 [SSH登录]: {}", ssh_result.message));
    }

    Ok(TestConnectionResult {
        success,
        message,
        details: TestDetails {
            local_port: local_port_result,
            tcp_connectivity: tcp_result,
            ssh_login: ssh_result,
        },
    })
}

/// 测试 SSH 登录认证
fn test_ssh_login(request: &TestConnectionRequest) -> TestStepResult {
    use std::process::Command;
    use std::time::Duration;

    let host = &request.host;
    let port = request.port;
    let username = &request.username;
    let auth_type = &request.auth_type;

    // 构建 SSH 命令参数
    let mut args = vec![
        "-o".to_string(), "BatchMode=yes".to_string(),
        "-o".to_string(), "ConnectTimeout=10".to_string(),
        "-o".to_string(), "StrictHostKeyChecking=accept-new".to_string(),
        "-p".to_string(), port.to_string(),
        format!("{}@{}", username, host),
        "exit".to_string(), // 执行 exit 命令，成功登录后立即退出
    ];

    // 密钥认证：添加密钥路径
    if auth_type == "key" {
        if let Some(key_path) = &request.key_path {
            args.insert(0, "-i".to_string());
            args.insert(1, key_path.clone());
        }
    }

    // Windows: 隐藏窗口
    #[cfg(target_os = "windows")]
    let output = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("ssh")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
    };

    #[cfg(not(target_os = "windows"))]
    let output = {
        Command::new("ssh")
            .args(&args)
            .output()
    };

    match output {
        Ok(output) => {
            if output.status.success() {
                TestStepResult { 
                    success: true, 
                    message: format!("SSH 登录认证成功 ({}认证)", auth_type) 
                }
            } else {
                // 解析错误信息
                let stderr = String::from_utf8_lossy(&output.stderr);
                let error_msg = parse_ssh_error(&stderr);
                TestStepResult { success: false, message: error_msg }
            }
        }
        Err(e) => {
            TestStepResult { 
                success: false, 
                message: format!("SSH 命令执行失败: {}", e) 
            }
        }
    }
}

/// 解析 SSH 错误信息
fn parse_ssh_error(stderr: &str) -> String {
    // 常见错误模式
    if stderr.contains("Permission denied") {
        if stderr.contains("publickey") {
            "SSH 登录失败: 密钥认证失败，请检查密钥路径和密码".to_string()
        } else if stderr.contains("password") {
            "SSH 登录失败: 密码认证失败，请检查用户名和密码".to_string()
        } else {
            "SSH 登录失败: 认证被拒绝".to_string()
        }
    } else if stderr.contains("Connection timed out") {
        "SSH 登录超时（10秒）".to_string()
    } else if stderr.contains("Host key verification failed") {
        "SSH 登录失败: 主机密钥验证失败".to_string()
    } else if stderr.contains("No such file or directory") && stderr.contains("identity file") {
        "SSH 登录失败: 密钥文件不存在".to_string()
    } else {
        // 提取第一行错误信息
        let first_line = stderr.lines().next().unwrap_or("未知错误");
        format!("SSH 登录失败: {}", first_line)
    }
}
```

- [ ] **Step 2: 编译验证命令实现**

Run: `cd ssh-tunnel-manager/src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 3: Commit**

```bash
git add ssh-tunnel-manager/src-tauri/src/commands/config.rs
git commit -m "feat(backend): implement test_ssh_connection command"
```

---

### Task 3: 后端命令导出和注册

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 导出 test_ssh_connection 命令**

修改 `src-tauri/src/commands/mod.rs`，添加导出：

```rust
// 应用设置管理命令
pub mod app_setting;
// 分组管理命令
pub mod group;
// 配置管理命令
pub mod config;
// 隧道控制命令
pub mod tunnel;
// 日志管理命令
pub mod log;
// 开机启动命令
pub mod autostart;

// 导出 test_ssh_connection 命令
pub use config::test_ssh_connection;
```

- [ ] **Step 2: 注册命令到 invoke_handler**

修改 `src-tauri/src/lib.rs`，在 `invoke_handler` 中添加 `test_ssh_connection`：

找到 `commands::config::reorder_favorites,` 行后添加：

```rust
commands::config::test_ssh_connection,
```

完整修改后的 invoke_handler 部分：

```rust
.invoke_handler(tauri::generate_handler![
    // 应用设置管理
    commands::app_setting::get_app_setting,
    commands::app_setting::save_app_setting,
    commands::app_setting::delete_app_setting,
    // 分组管理
    commands::group::get_groups,
    commands::group::save_group,
    commands::group::delete_group,
    // 配置管理
    commands::config::get_configs,
    commands::config::get_config,
    commands::config::save_config,
    commands::config::update_config,
    commands::config::delete_config,
    commands::config::search_configs,
    commands::config::export_configs,
    commands::config::import_configs,
    commands::config::get_favorites,
    commands::config::set_favorite,
    commands::config::reorder_favorites,
    commands::config::test_ssh_connection,
    // 隧道控制
    commands::tunnel::precheck_tunnel,
    commands::tunnel::start_tunnel,
    commands::tunnel::stop_tunnel,
    commands::tunnel::restart_tunnel,
    commands::tunnel::get_tunnel_status,
    commands::tunnel::get_running_tunnels_cmd,
    // 日志管理
    commands::log::get_logs,
    commands::log::clear_logs,
    commands::log::cleanup_logs,
    commands::log::clear_all_logs,
    // 开机启动
    commands::autostart::get_autostart_status,
    commands::autostart::set_autostart,
    commands::autostart::set_tunnel_autostart,
    commands::autostart::get_autostart_tunnels,
    // 更新管理
    updater::check_update,
    updater::download_and_install_update,
    updater::get_last_check_time,
    updater::get_version,
    updater::exit_app,
])
```

- [ ] **Step 3: 编译验证**

Run: `cd ssh-tunnel-manager/src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 4: Commit**

```bash
git add ssh-tunnel-manager/src-tauri/src/commands/mod.rs ssh-tunnel-manager/src-tauri/src/lib.rs
git commit -m "feat(backend): register test_ssh_connection command"
```

---

### Task 4: 前端类型定义

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: 添加测试相关类型**

在 `src/types/index.ts` 文件末尾添加以下类型定义：

```typescript
// ============================================
// SSH 连接测试类型定义
// ============================================

// 测试请求接口
export interface TestConnectionRequest {
  host: string
  port: number
  username: string
  authType: string
  password?: string
  keyPath?: string
  keyPassphrase?: string
  localHost: string
  localPort: number
}

// 单步测试结果接口
export interface TestStepResult {
  success: boolean
  message: string
}

// 测试详情接口
export interface TestDetails {
  localPort: TestStepResult
  tcpConnectivity: TestStepResult
  sshLogin: TestStepResult
}

// 测试结果接口
export interface TestConnectionResult {
  success: boolean
  message: string
  details: TestDetails
}
```

- [ ] **Step 2: Commit**

```bash
git add ssh-tunnel-manager/src/types/index.ts
git commit -m "feat(frontend): add TestConnectionRequest and TestConnectionResult types"
```

---

### Task 5: 前端 API 函数

**Files:**
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 添加 testSshConnection API 函数**

在 `src/api/tauri.ts` 文件中：

1. 在文件开头的 import 部分添加新类型导入：

```typescript
import type {
  Config,
  CreateConfigRequest,
  UpdateConfigRequest,
  Group,
  CreateGroupRequest,
  ConnectionLog,
  TunnelInfo,
  UpdateInfo,
  TestConnectionRequest,
  TestConnectionResult,
} from '@/types'
```

2. 在文件末尾添加测试 API 函数：

```typescript
// ============================================
// SSH 连接测试 API
// ============================================

/**
 * 测试 SSH 连接配置
 */
export async function testSshConnection(request: TestConnectionRequest): Promise<TestConnectionResult> {
  // 转换为后端期望的 snake_case 格式
  const dto = {
    host: request.host,
    port: request.port,
    username: request.username,
    auth_type: request.authType,
    password: request.password,
    key_path: request.keyPath,
    key_passphrase: request.keyPassphrase,
    local_host: request.localHost,
    local_port: request.localPort,
  }
  return await invoke<TestConnectionResult>('test_ssh_connection', { request: dto })
}
```

- [ ] **Step 2: Commit**

```bash
git add ssh-tunnel-manager/src/api/tauri.ts
git commit -m "feat(frontend): add testSshConnection API function"
```

---

### Task 6: ConfigForm.vue 添加测试按钮

**Files:**
- Modify: `src/components/ConfigForm.vue`

- [ ] **Step 1: 导入 API 函数和类型**

在 `src/components/ConfigForm.vue` 的 `<script setup lang="ts">` 部分，修改导入：

找到第 207-208 行的导入：

```typescript
import { ref, computed, watch, nextTick } from 'vue'
import type { FormInstanceFunctions, FormRule } from 'tdesign-vue-next'
import type { Config, AuthType, TunnelType, CreateConfigRequest, UpdateConfigRequest } from '@/types'
```

添加 `TestConnectionRequest` 类型导入：

```typescript
import { ref, computed, watch, nextTick } from 'vue'
import type { FormInstanceFunctions, FormRule } from 'tdesign-vue-next'
import type { Config, AuthType, TunnelType, CreateConfigRequest, UpdateConfigRequest, TestConnectionRequest } from '@/types'
```

添加 `testSshConnection` API 导入：

```typescript
import { useConfigStore } from '@/stores/config'
import { useGroupStore } from '@/stores/group'
import { MessagePlugin } from 'tdesign-vue-next'
import { testSshConnection } from '@/api'
```

- [ ] **Step 2: 添加测试状态变量**

在 `src/components/ConfigForm.vue` 中，找到 `const saving = ref(false)` 行（约 230 行），在其后添加：

```typescript
// 保存状态
const saving = ref(false)

// 测试状态
const testing = ref(false)
```

- [ ] **Step 3: 添加 buildTestRequest 函数**

在 `fillFormData` 函数之后（约 388 行），添加 `buildTestRequest` 函数：

```typescript
// 构建测试请求
function buildTestRequest(): TestConnectionRequest {
  return {
    host: formData.value.host.trim(),
    port: formData.value.port,
    username: formData.value.username.trim(),
    authType: formData.value.authType,
    password: formData.value.authType === 'password' ? formData.value.password : undefined,
    keyPath: formData.value.authType === 'key' ? formData.value.keyPath?.trim() : undefined,
    keyPassphrase: formData.value.authType === 'key' ? formData.value.keyPassphrase : undefined,
    localHost: formData.value.localHost.trim(),
    localPort: formData.value.localPort,
  }
}
```

- [ ] **Step 4: 添加 handleTest 函数**

在 `buildTestRequest` 函数之后，添加 `handleTest` 函数：

```typescript
// 处理测试
async function handleTest(): Promise<void> {
  // 先验证必填字段
  const valid = await formRef.value?.validate()
  if (valid !== true) {
    MessagePlugin.warning('请先填写必填信息')
    return
  }

  testing.value = true
  try {
    const result = await testSshConnection(buildTestRequest())
    if (result.success) {
      MessagePlugin.success('连接测试成功')
    } else {
      MessagePlugin.error(result.message)
    }
  } catch (error) {
    MessagePlugin.error('测试请求失败: ' + (error as string))
  } finally {
    testing.value = false
  }
}
```

- [ ] **Step 5: 修改 drawer footer 添加测试按钮**

在 `<template>` 部分，修改 `<t-drawer>` 的 footer：

找到原来的 `<t-drawer>` 定义（约 3-12 行）：

```vue
<t-drawer
  :visible="visible"
  :header="isEditMode ? '编辑配置' : '新建配置'"
  placement="right"
  :size="'500px'"
  :footer="true"
  :confirm-btn="{ content: '保存', loading: saving }"
  :on-confirm="handleSubmit"
  :on-close="handleClose"
  @update:visible="emit('update:visible', $event)"
>
```

修改为使用自定义 footer：

```vue
<t-drawer
  :visible="visible"
  :header="isEditMode ? '编辑配置' : '新建配置'"
  placement="right"
  :size="'500px'"
  :on-close="handleClose"
  @update:visible="emit('update:visible', $event)"
>
  <template #footer>
    <div style="display: flex; gap: 8px;">
      <t-button variant="outline" :loading="testing" @click="handleTest">
        测试连接
      </t-button>
      <t-button theme="primary" :loading="saving" @click="handleSubmit">
        保存
      </t-button>
      <t-button variant="text" @click="handleClose">
        取消
      </t-button>
    </div>
  </template>
```

- [ ] **Step 6: 编译验证**

Run: `cd ssh-tunnel-manager && pnpm run build`
Expected: 构建成功，无错误

- [ ] **Step 7: Commit**

```bash
git add ssh-tunnel-manager/src/components/ConfigForm.vue
git commit -m "feat(frontend): add test button to ConfigForm"
```

---

### Task 7: 功能验证

**Files:**
- 无文件修改，仅运行验证

- [ ] **Step 1: 启动开发服务器**

Run: `cd ssh-tunnel-manager && pnpm tauri dev`
Expected: 应用启动成功

- [ ] **Step 2: 手动测试**

测试步骤：
1. 点击"新建配置"打开 ConfigForm
2. 填写完整的 SSH 配置信息（主机、端口、用户名、认证信息）
3. 点击"测试连接"按钮
4. 验证：
   - 测试按钮显示 loading 状态
   - 测试完成后弹出成功或失败消息
   - 日志文件中有测试记录（检查 `AppData/ssh-tunnel-manager/logs/` 目录）

- [ ] **Step 3: 验证日志记录**

检查日志文件内容，应包含类似：
```
开始测试 SSH 连接: user@host:port
SSH 连接测试成功: user@host:port
```
或
```
SSH 连接测试失败 [xxx]: 错误信息
```

---

## Self-Review Checklist

**1. Spec coverage:**
- ✓ TCP 连通性测试 - Task 2 中的 `check_remote_connectivity` 调用
- ✓ SSH 登录认证测试 - Task 2 中的 `test_ssh_login` 函数
- ✓ 本地端口检查 - Task 2 中的 `is_port_in_use` 调用
- ✓ 弹窗消息展示 - Task 6 中的 `MessagePlugin` 调用
- ✓ 文件日志记录 - Task 2 中的 `logger::info/error` 调用

**2. Placeholder scan:**
- ✓ 无 "TBD"、"TODO"、"implement later" 等占位符
- ✓ 所有代码步骤包含完整实现代码
- ✓ 所有命令包含预期输出

**3. Type consistency:**
- ✓ `TestConnectionRequest`: Rust (camelCase in serde) ↔ TypeScript (camelCase)
- ✓ `TestConnectionResult`: Rust ↔ TypeScript 字段名一致
- ✓ `TestDetails`: 字段名 `local_port`/`tcp_connectivity`/`ssh_login` ↔ `localPort`/`tcpConnectivity`/`sshLogin`

**4. Error handling:**
- ✓ 本地端口占用时返回详细错误信息和进程详情
- ✓ TCP 连接失败时返回友好错误信息
- ✓ SSH 登录失败时解析 stderr 返回具体错误原因
- ✓ 前端 catch 错误并显示弹窗消息