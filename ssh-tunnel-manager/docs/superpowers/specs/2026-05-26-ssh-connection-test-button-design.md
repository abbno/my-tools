# SSH 连接测试按钮功能设计

## 概述

在 SSH Tunnel Manager 的配置编辑弹框中添加"测试"按钮，用于验证 SSH 连接配置是否有效。

## 功能需求

1. 测试 TCP 连通性 - 验证服务器端口是否可达
2. 测试 SSH 登录认证 - 验证用户名和密码/密钥是否能成功登录
3. 检查本地端口占用 - 验证本地监听端口是否可用
4. 显示测试结果 - 使用弹窗消息展示成功/失败及具体原因
5. 记录日志 - 将测试过程记录到文件日志

## 设计决策

| 决策项 | 选择 | 原因 |
|--------|------|------|
| 测试范围 | TCP + SSH 登录 | 全面验证配置有效性 |
| 按钮位置 | Footer 区域，保存按钮旁 | 操作便捷，符合用户习惯 |
| 结果展示 | 弹窗消息 (MessagePlugin) | 简洁直观 |
| 日志记录 | 仅文件日志 | 不干扰 UI 连接日志面板 |
| SSH 登录超时 | 10 秒 | 兼顾用户体验和网络延迟 |

## 整体架构

### 数据流

```
用户填写表单 → 点击"测试"按钮 → 前端调用 test_ssh_connection 命令
    → 后端执行：
       1. 检查本地端口占用
       2. TCP 连通性测试（2秒超时）
       3. SSH 登录认证测试（10秒超时）
    → 返回测试结果 → 前端显示弹窗消息
```

### 新增组件

| 层级 | 新增内容 |
|------|----------|
| 后端 | `test_ssh_connection` Tauri 命令 |
| 后端 | `TestConnectionRequest` 请求结构体 |
| 后端 | `TestConnectionResult` 结果结构体 |
| 前端 | `ConfigForm.vue` 添加测试按钮 |
| 前端 | `api/tauri.ts` 新增 API 函数 |

## 后端实现

### 文件位置

`src-tauri/src/commands/config.rs`

### 新增类型定义

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

/// 测试结果
#[derive(Debug, Serialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub message: String,
    pub details: TestDetails,
}

#[derive(Debug, Serialize)]
pub struct TestDetails {
    pub tcp_connectivity: TestStepResult,
    pub ssh_login: TestStepResult,
    pub local_port: TestStepResult,
}

#[derive(Debug, Serialize)]
pub struct TestStepResult {
    pub success: bool,
    pub message: String,
}
```

### Tauri 命令

```rust
#[tauri::command]
pub async fn test_ssh_connection(request: TestConnectionRequest) -> Result<TestConnectionResult, String>
```

### 测试逻辑执行顺序

1. **本地端口检查** - 调用现有 `is_port_in_use` 和 `get_port_process_info`
2. **TCP 连通性** - 调用现有 `check_remote_connectivity`（2秒超时）
3. **SSH 登录** - 执行 SSH 命令验证认证（10秒超时）

### SSH 登录测试实现

使用系统 SSH 命令执行快速登录测试：

```rust
// 密码认证：使用 sshpass 或直接尝试（密码认证无法通过 BatchMode）
// 密钥认证：ssh -o BatchMode=yes -o ConnectTimeout=10 -i key_path user@host exit
```

对于密码认证，由于 SSH 默认不接受命令行密码输入，需要特殊处理：
- 方案：执行 `ssh -o PreferredAuthentications=password -o PubkeyAuthentication=no` 并观察输出
- 若密码错误，SSH 会返回 "Permission denied" 错误

## 前端实现

### ConfigForm.vue 修改

**1. Footer 区域自定义**

替换默认 footer，添加测试按钮：

```vue
<template #footer>
  <div class="drawer-footer">
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

**2. 新增状态和函数**

```typescript
const testing = ref(false)

async function handleTest() {
  // 验证必填字段
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
    MessagePlugin.error('测试失败: ' + (error as string))
  } finally {
    testing.value = false
  }
}

function buildTestRequest(): TestConnectionRequest {
  return {
    host: formData.value.host.trim(),
    port: formData.value.port,
    username: formData.value.username.trim(),
    auth_type: formData.value.authType,
    password: formData.value.authType === 'password' ? formData.value.password : null,
    key_path: formData.value.authType === 'key' ? formData.value.keyPath.trim() : null,
    key_passphrase: formData.value.authType === 'key' ? formData.value.keyPassphrase : null,
    local_host: formData.value.localHost.trim(),
    local_port: formData.value.localPort,
  }
}
```

### API 函数

**文件**: `src/api/tauri.ts`

```typescript
export interface TestConnectionRequest {
  host: string
  port: number
  username: string
  auth_type: string
  password?: string
  key_path?: string
  key_passphrase?: string
  local_host: string
  local_port: number
}

export interface TestConnectionResult {
  success: boolean
  message: string
  details: TestDetails
}

export interface TestDetails {
  tcp_connectivity: TestStepResult
  ssh_login: TestStepResult
  local_port: TestStepResult
}

export interface TestStepResult {
  success: boolean
  message: string
}

export async function testSshConnection(request: TestConnectionRequest): Promise<TestConnectionResult> {
  return await invoke<TestConnectionResult>('test_ssh_connection', { request })
}
```

### 类型定义

**文件**: `src/types/index.ts`

新增上述接口定义。

## 错误处理

### 测试结果消息格式

| 场景 | 消息示例 |
|------|----------|
| 全部成功 | `连接测试成功` |
| 端口被占用 | `本地端口 127.0.0.1:8080 已被占用 (PID: 1234, 进程: nginx)` |
| TCP 连接失败 | `远程主机 192.168.1.100:22 连接失败: Connection refused` |
| SSH 认证失败 | `SSH 登录失败: Permission denied (publickey,password)` |
| SSH 超时 | `SSH 登录测试超时（10秒）` |

### 错误边界

- **超时处理**：SSH 登录超过 10 秒返回超时错误
- **网络异常**：捕获 TCP 连接错误并友好提示
- **进程异常**：SSH 命令执行失败时解析 stderr 输出

## 日志记录

使用现有 `logger::info/error` 记录到文件：

```rust
// 测试开始
logger::info(&format!("开始测试 SSH 连接: {}@{}:{}", username, host, port));

// 测试成功
logger::info(&format!("SSH 连接测试成功: {}@{}:{}", username, host, port));

// 测试失败
logger::error(&format!("SSH 连接测试失败 [{}]: {}", step, error_msg));
```

日志文件位置：应用数据目录下的日志文件。

## 文件修改清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/commands/config.rs` | 修改 | 新增测试命令和类型 |
| `src-tauri/src/commands/mod.rs` | 修改 | 导出新命令 |
| `src-tauri/src/lib.rs` | 修改 | 注册新命令 |
| `src/api/tauri.ts` | 修改 | 新增 API 函数和类型 |
| `src/types/index.ts` | 修改 | 新增类型定义 |
| `src/components/ConfigForm.vue` | 修改 | 添加测试按钮和处理逻辑 |