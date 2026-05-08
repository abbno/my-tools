// SSH Tunnel Manager - Tauri API 封装
// 提供类型安全的 API 调用接口

import { invoke } from '@tauri-apps/api/core'
import type {
  Config,
  CreateConfigRequest,
  UpdateConfigRequest,
  Group,
  CreateGroupRequest,
  ConnectionLog,
  TunnelInfo,
  UpdateInfo,
} from '@/types'

// ============================================
// 工具函数：camelCase <-> snake_case 转换
// ============================================

/**
 * 将 camelCase 转换为 snake_case
 */
// function toSnakeCase(str: string): string {
//   return str.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`)
// }

/**
 * 将对象的键从 camelCase 转换为 snake_case（未使用）
 */
// function keysToSnakeCase<T>(obj: T): Record<string, unknown> {
//   if (obj === null || obj === undefined) {
//     return obj as Record<string, unknown>
//   }
//   if (Array.isArray(obj)) {
//     return obj.map(keysToSnakeCase) as unknown as Record<string, unknown>
//   }
//   if (typeof obj !== 'object') {
//     return obj as Record<string, unknown>
//   }
//   const result: Record<string, unknown> = {}
//   for (const [key, value] of Object.entries(obj)) {
//     result[toSnakeCase(key)] = value
//   }
//   return result
// }

/**
 * 将 snake_case 转换为 camelCase
 */
// function toCamelCase(str: string): string {
//   return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase())
// }

/**
 * 将对象的键从 snake_case 转换为 camelCase（未使用）
 */
// function keysToCamelCase<T>(obj: T): T {
//   if (obj === null || obj === undefined) {
//     return obj
//   }
//   if (Array.isArray(obj)) {
//     return (obj as unknown[]).map(keysToCamelCase) as T
//   }
//   if (typeof obj !== 'object') {
//     return obj
//   }
//   const result: Record<string, unknown> = {}
//   for (const [key, value] of Object.entries(obj as Record<string, unknown>)) {
//     result[toCamelCase(key)] = keysToCamelCase(value)
//   }
//   return result as T
// }

// ============================================
// DTO 类型定义（与后端对应）
// ============================================

interface GroupDto {
  id: string
  name: string
  sort_order: number
  created_at: string
}

interface ConfigDto {
  id: string
  name: string
  group_id: string | null
  host: string
  port: number
  username: string
  auth_type: string
  password: string | null
  key_path: string | null
  key_passphrase: string | null
  tunnel_type: string
  local_host: string
  local_port: number
  remote_host: string | null
  remote_port: number | null
  auto_reconnect: boolean
  reconnect_interval: number
  is_favorite: boolean
  favorite_order: number
  auto_start: boolean
  created_at: string
  updated_at: string
}

interface LogDto {
  id: string
  config_id: string
  action: string
  message: string
  created_at: string
}

interface TunnelStatusDto {
  config_id: string
  status: string
  pid: number | null
  message: string | null
}

// ============================================
// DTO 转换函数
// ============================================

function groupDtoToGroup(dto: GroupDto): Group {
  return {
    id: dto.id,
    name: dto.name,
    sortOrder: dto.sort_order,
    createdAt: dto.created_at,
  }
}

function configDtoToConfig(dto: ConfigDto): Config {
  return {
    id: dto.id,
    name: dto.name,
    groupId: dto.group_id,
    host: dto.host,
    port: dto.port,
    username: dto.username,
    authType: dto.auth_type as Config['authType'],
    password: dto.password,
    keyPath: dto.key_path,
    keyPassphrase: dto.key_passphrase,
    tunnelType: dto.tunnel_type as Config['tunnelType'],
    localHost: dto.local_host,
    localPort: dto.local_port,
    remoteHost: dto.remote_host,
    remotePort: dto.remote_port,
    autoReconnect: dto.auto_reconnect,
    reconnectInterval: dto.reconnect_interval,
    isFavorite: dto.is_favorite,
    favoriteOrder: dto.favorite_order,
    autoStart: dto.auto_start,
    createdAt: dto.created_at,
    updatedAt: dto.updated_at,
  }
}

function createConfigRequestToDto(req: CreateConfigRequest): Record<string, unknown> {
  return {
    name: req.name,
    group_id: req.groupId,
    host: req.host,
    port: req.port,
    username: req.username,
    auth_type: req.authType,
    password: req.password,
    key_path: req.keyPath,
    key_passphrase: req.keyPassphrase,
    tunnel_type: req.tunnelType,
    local_host: req.localHost,
    local_port: req.localPort,
    remote_host: req.remoteHost,
    remote_port: req.remotePort,
    auto_reconnect: req.autoReconnect,
    reconnect_interval: req.reconnectInterval,
    is_favorite: req.isFavorite ?? false,
    auto_start: req.autoStart ?? false,
  }
}

function updateConfigRequestToDto(req: UpdateConfigRequest): Record<string, unknown> {
  return {
    id: req.id,
    ...createConfigRequestToDto(req),
  }
}

function logDtoToConnectionLog(dto: LogDto): ConnectionLog {
  return {
    id: dto.id,
    configId: dto.config_id,
    action: dto.action as ConnectionLog['action'],
    message: dto.message,
    createdAt: dto.created_at,
  }
}

function tunnelStatusDtoToTunnelInfo(dto: TunnelStatusDto): TunnelInfo {
  return {
    configId: dto.config_id,
    status: dto.status as TunnelInfo['status'],
    pid: dto.pid,
    errorMessage: dto.message,
  }
}

function createGroupRequestToDto(req: CreateGroupRequest): Record<string, unknown> {
  return {
    name: req.name,
    sort_order: req.sortOrder,
  }
}

// ============================================
// 分组管理 API
// ============================================

/**
 * 获取所有分组
 */
export async function getGroups(): Promise<Group[]> {
  const dtos = await invoke<GroupDto[]>('get_groups')
  return dtos.map(groupDtoToGroup)
}

/**
 * 保存分组（创建新分组）
 */
export async function saveGroup(request: CreateGroupRequest): Promise<Group> {
  const dto = await invoke<GroupDto>('save_group', { request: createGroupRequestToDto(request) })
  return groupDtoToGroup(dto)
}

/**
 * 删除分组
 */
export async function deleteGroup(id: string): Promise<void> {
  await invoke('delete_group', { id })
}

// ============================================
// 配置管理 API
// ============================================

/**
 * 获取配置列表
 * @param groupId 可选，指定分组ID则只返回该分组的配置
 */
export async function getConfigs(groupId?: string): Promise<Config[]> {
  const dtos = await invoke<ConfigDto[]>('get_configs', { groupId })
  return dtos.map(configDtoToConfig)
}

/**
 * 获取单个配置
 */
export async function getConfig(id: string): Promise<Config | null> {
  const dto = await invoke<ConfigDto | null>('get_config', { id })
  return dto ? configDtoToConfig(dto) : null
}

/**
 * 创建配置
 */
export async function saveConfig(request: CreateConfigRequest): Promise<Config> {
  const dto = await invoke<ConfigDto>('save_config', { request: createConfigRequestToDto(request) })
  return configDtoToConfig(dto)
}

/**
 * 更新配置
 */
export async function updateConfig(request: UpdateConfigRequest): Promise<Config> {
  const dto = await invoke<ConfigDto>('update_config', { request: updateConfigRequestToDto(request) })
  return configDtoToConfig(dto)
}

/**
 * 删除配置
 */
export async function deleteConfig(id: string): Promise<void> {
  await invoke('delete_config', { id })
}

/**
 * 搜索配置
 */
export async function searchConfigs(keyword: string): Promise<Config[]> {
  const dtos = await invoke<ConfigDto[]>('search_configs', { keyword })
  return dtos.map(configDtoToConfig)
}

/**
 * 获取常用配置列表
 */
export async function getFavorites(): Promise<Config[]> {
  const dtos = await invoke<ConfigDto[]>('get_favorites')
  return dtos.map(configDtoToConfig)
}

/**
 * 设置配置的常用状态
 */
export async function setFavorite(configId: string, isFavorite: boolean): Promise<Config> {
  const dto = await invoke<ConfigDto>('set_favorite', { configId, isFavorite })
  return configDtoToConfig(dto)
}

/**
 * 批量更新常用配置排序
 * @param orders 排序数据数组，每项包含 configId 和 order
 */
export async function reorderFavorites(orders: { configId: string; order: number }[]): Promise<void> {
  // 转换为后端期望的元组格式
  const ordersTuple: [string, number][] = orders.map(o => [o.configId, o.order])
  await invoke('reorder_favorites', { orders: ordersTuple })
}

// ============================================
// 隧道控制 API
// ============================================

// 预检查结果类型
export interface PreCheckResult {
  remoteOk: boolean
  remoteError: string | null
  localPortOk: boolean
  localPortError: string | null
  portProcessInfo: {
    pid: number
    name: string
  } | null
}

/**
 * 预检查隧道启动条件
 */
export async function precheckTunnel(configId: string): Promise<PreCheckResult> {
  return await invoke<PreCheckResult>('precheck_tunnel', { configId })
}

/**
 * 启动隧道
 */
export async function startTunnel(configId: string): Promise<TunnelInfo> {
  const dto = await invoke<TunnelStatusDto>('start_tunnel', { configId })
  return tunnelStatusDtoToTunnelInfo(dto)
}

/**
 * 停止隧道
 */
export async function stopTunnel(configId: string): Promise<TunnelInfo> {
  const dto = await invoke<TunnelStatusDto>('stop_tunnel', { configId })
  return tunnelStatusDtoToTunnelInfo(dto)
}

/**
 * 重启隧道
 */
export async function restartTunnel(configId: string): Promise<TunnelInfo> {
  const dto = await invoke<TunnelStatusDto>('restart_tunnel', { configId })
  return tunnelStatusDtoToTunnelInfo(dto)
}

/**
 * 获取隧道状态
 */
export async function getTunnelStatus(configId: string): Promise<TunnelInfo> {
  const dto = await invoke<TunnelStatusDto>('get_tunnel_status', { configId })
  return tunnelStatusDtoToTunnelInfo(dto)
}

/**
 * 获取所有运行中的隧道
 */
export async function getRunningTunnels(): Promise<TunnelInfo[]> {
  const dtos = await invoke<TunnelStatusDto[]>('get_running_tunnels_cmd')
  return dtos.map(tunnelStatusDtoToTunnelInfo)
}

// ============================================
// 日志管理 API
// ============================================

/**
 * 获取指定配置的连接日志
 * @param configId 配置ID
 * @param limit 可选，限制返回数量
 */
export async function getLogs(configId: string, limit?: number): Promise<ConnectionLog[]> {
  const dtos = await invoke<LogDto[]>('get_logs', { configId, limit })
  return dtos.map(logDtoToConnectionLog)
}

/**
 * 清除指定配置的日志
 */
export async function clearLogs(configId: string): Promise<void> {
  await invoke('clear_logs', { configId })
}

/**
 * 清理超过指定天数的旧日志（默认30天）
 * @param days 保留最近多少天的日志
 * @returns 删除的日志数量
 */
export async function cleanupLogs(days?: number): Promise<number> {
  return await invoke<number>('cleanup_logs', { days })
}

/**
 * 清除所有日志
 */
export async function clearAllLogs(): Promise<void> {
  await invoke('clear_all_logs')
}

// ============================================
// 导入导出 API
// ============================================

/**
 * 导出配置
 * @param ids 可选，指定导出的配置ID列表，不指定则导出全部
 * @returns JSON 字符串
 */
export async function exportConfigs(ids?: string[]): Promise<string> {
  return await invoke<string>('export_configs', { ids })
}

/**
 * 导入配置
 * @param json JSON 字符串
 * @returns 导入的配置数量
 */
export async function importConfigs(json: string): Promise<number> {
  return await invoke<number>('import_configs', { json })
}

// ============================================
// 开机启动 API
// ============================================

/**
 * 获取软件开机启动状态
 */
export async function getAutostartStatus(): Promise<boolean> {
  return await invoke<boolean>('get_autostart_status')
}

/**
 * 设置软件开机启动状态
 */
export async function setAutostart(enable: boolean): Promise<void> {
  await invoke('set_autostart', { enable })
}

/**
 * 设置隧道开机启动状态
 */
export async function setTunnelAutostart(configId: string, autoStart: boolean): Promise<Config> {
  const dto = await invoke<ConfigDto>('set_tunnel_autostart', { configId, autoStart })
  return configDtoToConfig(dto)
}

/**
 * 获取所有开机启动的隧道配置
 */
export async function getAutostartTunnels(): Promise<Config[]> {
  const dtos = await invoke<ConfigDto[]>('get_autostart_tunnels')
  return dtos.map(configDtoToConfig)
}

// ============================================
// 更新管理 API
// ============================================

interface UpdateInfoDto {
  version: string
  release_date: string
  changelog: Array<{ type: string; description: string }>
  download_url: string
  force_update: boolean
}

function updateInfoDtoToUpdateInfo(dto: UpdateInfoDto): UpdateInfo {
  return {
    version: dto.version,
    releaseDate: dto.release_date,
    changelog: dto.changelog.map(item => ({
      type: item.type as UpdateInfo['changelog'][0]['type'],
      description: item.description,
    })),
    downloadUrl: dto.download_url,
    forceUpdate: dto.force_update,
  }
}

/**
 * 检查更新
 */
export async function checkUpdate(): Promise<UpdateInfo | null> {
  const dto = await invoke<UpdateInfoDto | null>('check_update')
  return dto ? updateInfoDtoToUpdateInfo(dto) : null
}

/**
 * 下载并安装更新
 * 注意：调用此方法后应用会自动退出并重启
 */
export async function downloadAndInstallUpdate(): Promise<void> {
  await invoke('download_and_install_update')
}

/**
 * 获取上次检查时间
 */
export async function getLastCheckTime(): Promise<string | null> {
  return await invoke<string | null>('get_last_check_time')
}

// ============================================
// 应用设置 API
// ============================================

/**
 * 获取单个应用设置
 */
export async function getAppSetting(key: string): Promise<string | null> {
  return await invoke<string | null>('get_app_setting', { key })
}

/**
 * 保存应用设置
 */
export async function saveAppSetting(key: string, value: string): Promise<void> {
  await invoke('save_app_setting', { key, value })
}

/**
 * 删除应用设置
 */
export async function deleteAppSetting(key: string): Promise<void> {
  await invoke('delete_app_setting', { key })
}
