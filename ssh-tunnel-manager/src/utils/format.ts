import type { Config, TunnelType } from '@/types'

/**
 * 格式化日期时间
 */
export function formatDateTime(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

/**
 * 格式化 SSH 命令
 */
export function formatSshCommand(config: Config): string {
  const parts = ['ssh']

  switch (config.tunnelType) {
    case 'local':
      parts.push('-L')
      parts.push(`${config.localHost}:${config.localPort}:${config.remoteHost || 'localhost'}:${config.remotePort || 22}`)
      break
    case 'remote':
      parts.push('-R')
      parts.push(`${config.remoteHost || '0.0.0.0'}:${config.remotePort || 22}:${config.localHost}:${config.localPort}`)
      break
    case 'dynamic':
      parts.push('-D')
      parts.push(`${config.localHost}:${config.localPort}`)
      break
  }

  parts.push('-p')
  parts.push(String(config.port))
  parts.push(`${config.username}@${config.host}`)

  return parts.join(' ')
}

/**
 * 获取隧道类型显示名称
 */
export function getTunnelTypeLabel(type: TunnelType): string {
  const labels: Record<TunnelType, string> = {
    local: '本地转发',
    remote: '远程转发',
    dynamic: '动态转发'
  }
  return labels[type]
}

/**
 * 获取隧道类型标签颜色
 */
export function getTunnelTypeTheme(type: TunnelType): 'primary' | 'success' | 'warning' {
  const themes: Record<TunnelType, 'primary' | 'success' | 'warning'> = {
    local: 'primary',
    remote: 'success',
    dynamic: 'warning'
  }
  return themes[type]
}

/**
 * 复制文本到剪贴板
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    return false
  }
}
