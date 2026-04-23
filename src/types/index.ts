export interface Project {
  id: string
  name: string
  dir_name: string
  path: string
  git_url: string
  version: string
  branch: string
  port: number | null
  is_favorite: boolean
  last_run_time: string
  last_build_time: string
  scripts: Record<string, string>
  framework: string
  custom_dev_command: string
  custom_build_command: string
  sort_order: number
}

export interface AppConfig {
  workspace_folders: string[]
  projects: Project[]
  custom_names: Record<string, string>
  favorites: string[]
  ide_command: string
  package_manager: string
  dev_script: string
  build_script: string
}

export interface OutdatedDep {
  name: string
  current: string
  latest: string
  dep_type: string
}

export type ViewMode = 'table' | 'card'
export type FilterMode = 'all' | 'favorites'
