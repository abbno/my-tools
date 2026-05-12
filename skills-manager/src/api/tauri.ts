import { invoke } from '@tauri-apps/api/core'

export interface GitStatus {
  installed: boolean
  version: string | null
  path: string | null
}

export interface SystemInfo {
  os: string
  home_dir: string
  skill_manager_dir: string
}

export interface SkillMeta {
  id: string
  repo_id: string
  name: string
  description: string
  path: string
  local_path: string
  is_selected: boolean
}

export interface AuthConfig {
  type: 'none' | 'token' | 'username-password'
  token?: string
  username?: string
  password?: string
}

export interface Repository {
  id: string
  name: string
  url: string
  branch: string
  auth: AuthConfig
  sync_interval: number
  selected_skills: string[]
  last_sync: string | null
  enabled: boolean
}

export interface Agent {
  id: string
  name: string
  path: string
  enabled: boolean
}

export interface Settings {
  default_sync_interval: number
  auto_sync: boolean
  check_interval: number
}

export interface Config {
  repositories: Repository[]
  agents: Agent[]
  settings: Settings
}

// System commands
export async function checkGitInstalled(): Promise<GitStatus> {
  return invoke<GitStatus>('check_git_installed')
}

export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>('get_system_info')
}

// Repo commands
export async function fetchBranches(url: string, auth: AuthConfig): Promise<string[]> {
  return invoke<string[]>('fetch_branches', { url, auth })
}

export async function fetchRepoSkills(url: string, branch: string, auth: AuthConfig): Promise<SkillMeta[]> {
  return invoke<SkillMeta[]>('fetch_repo_skills', { url, branch, auth })
}

export async function syncRepository(
  repoId: string,
  url: string,
  branch: string,
  auth: AuthConfig,
  selectedSkills: string[] = []
): Promise<SkillMeta[]> {
  return invoke<SkillMeta[]>('sync_repository', { repoId, url, branch, auth, selectedSkills })
}

// SQLite skills API
export async function getSkills(repoId: string): Promise<SkillMeta[]> {
  return invoke<SkillMeta[]>('get_skills', { repoId })
}

export async function updateSkillSelection(skillId: string, isSelected: boolean): Promise<void> {
  return invoke<void>('update_skill_selection', { skillId, isSelected })
}

// Symlink commands
export interface SymlinkStatus {
  skill_name: string
  agent_id: string
  agent_path: string
  exists: boolean
  is_symlink: boolean
  target: string | null
}

export async function createSkillSymlink(repoId: string, skillName: string, agentPath: string): Promise<void> {
  return invoke<void>('create_skill_symlink', { repoId, skillName, agentPath })
}

export async function removeSkillSymlink(skillName: string, agentPath: string): Promise<void> {
  return invoke<void>('remove_skill_symlink', { skillName, agentPath })
}

export async function checkSymlinks(agents: [string, string][]): Promise<SymlinkStatus[]> {
  return invoke<SymlinkStatus[]>('check_symlinks', { agents })
}

// Deploy commands
export async function deploySkill(repoId: string, skillName: string, agentPaths: string[]): Promise<void> {
  return invoke<void>('deploy_skill', { repoId, skillName, agentPaths })
}

export async function undeploySkill(skillName: string, agentPaths: string[]): Promise<void> {
  return invoke<void>('undeploy_skill', { skillName, agentPaths })
}

// Sync commands
export async function syncAllRepositories(): Promise<string[]> {
  return invoke<string[]>('sync_all_repositories')
}

export async function getSyncStatus(): Promise<string> {
  return invoke<string>('get_sync_status')
}

// Config commands
export async function readConfig(): Promise<Config> {
  return invoke<Config>('read_config')
}

export async function saveConfig(config: Config): Promise<void> {
  return invoke<void>('save_config', { config })
}