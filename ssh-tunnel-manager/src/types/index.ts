// SSH Tunnel Manager - 前端类型定义
// 与 Rust 后端模型保持一致

// 认证类型
export type AuthType = 'password' | 'key'

// 隧道类型
export type TunnelType = 'local' | 'remote' | 'dynamic'

// 阧道状态
export type TunnelStatus = 'stopped' | 'starting' | 'running' | 'stopping' | 'error' | 'reconnecting'

// 日志操作类型
export type LogAction = 'connect' | 'disconnect' | 'reconnect' | 'error'

// 配置接口
export interface Config {
  id: string
  name: string
  groupId: string | null
  host: string
  port: number
  username: string
  authType: AuthType
  password: string | null
  keyPath: string | null
  keyPassphrase: string | null
  tunnelType: TunnelType
  localHost: string
  localPort: number
  remoteHost: string | null
  remotePort: number | null
  autoReconnect: boolean
  reconnectInterval: number
  isFavorite: boolean
  favoriteOrder: number
  autoStart: boolean
  createdAt: string
  updatedAt: string
}

// 创建配置请求
export interface CreateConfigRequest {
  name: string
  groupId: string | null
  host: string
  port: number
  username: string
  authType: AuthType
  password: string | null
  keyPath: string | null
  keyPassphrase: string | null
  tunnelType: TunnelType
  localHost: string
  localPort: number
  remoteHost: string | null
  remotePort: number | null
  autoReconnect: boolean
  reconnectInterval: number
  isFavorite?: boolean
  autoStart?: boolean
}

// 更新配置请求
export interface UpdateConfigRequest extends CreateConfigRequest {
  id: string
}

// 分组接口
export interface Group {
  id: string
  name: string
  sortOrder: number
  createdAt: string
}

// 创建分组请求
export interface CreateGroupRequest {
  name: string
  sortOrder?: number
}

// 更新分组请求
export interface UpdateGroupRequest {
  id: string
  name: string
  sortOrder: number
}

// 连接日志接口
export interface ConnectionLog {
  id: string
  configId: string
  action: LogAction
  message: string
  createdAt: string
}

// 阧道信息接口
export interface TunnelInfo {
  configId: string
  status: TunnelStatus
  pid: number | null
  errorMessage: string | null
}

// 端口占用进程信息
export interface PortProcessInfo {
  pid: number
  name: string
}

// 预检查结果接口
export interface PreCheckResult {
  remoteOk: boolean
  remoteError: string | null
  localPortOk: boolean
  localPortError: string | null
  portProcessInfo: PortProcessInfo | null
}

// 更新信息接口
export interface UpdateInfo {
  version: string
  releaseDate: string
  changelog: ChangelogItem[]
  downloadUrl: string
  fullDownloadUrl: string // 完整下载地址，用于前端显示
  forceUpdate: boolean
}

// 更新日志项接口
export interface ChangelogItem {
  type: 'feature' | 'fix' | 'improve'
  description: string
}

// 下载进度接口
export interface DownloadProgress {
  downloaded: number
  total: number
  percentage: number
}

// ============================================
// 应用设置类型定义
// ============================================

// 应用设置项
export interface AppSetting {
  key: string
  value: string
  updatedAt: string
}

// 保存设置请求
export interface SaveAppSettingRequest {
  key: string
  value: string
}

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