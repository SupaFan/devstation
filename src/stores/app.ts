import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, Project, OutdatedDep, ViewMode, FilterMode } from '../types'

export const useAppStore = defineStore('app', () => {
  const config = ref<AppConfig>({
    workspace_folders: [],
    projects: [],
    custom_names: {},
    favorites: [],
    ide_command: 'Trae',
    package_manager: 'pnpm',
    dev_script: 'dev',
    build_script: 'build',
  })
  const loading = ref(false)
  const searchQuery = ref('')
  const viewMode = ref<ViewMode>('table')
  const filterMode = ref<FilterMode>('all')
  const currentView = ref<'main' | 'settings'>('main')
  const outdatedCache = ref<Record<string, OutdatedDep[]>>({})
  const selectedIds = ref<string[]>([])
  const runningPorts = ref<Set<number>>(new Set())

  async function loadConfig() {
    try {
      config.value = await invoke<AppConfig>('load_config')
      // First time: assign sort_order by current order
      if (config.value.projects.length > 0 && config.value.projects.every(p => p.sort_order === 0)) {
        config.value.projects.forEach((p, i) => { p.sort_order = i + 1 })
        await saveConfig()
      }
    } catch (e) {
      console.error('加载配置失败:', e)
    }
  }

  async function saveConfig() {
    try {
      await invoke('save_config', { config: config.value })
    } catch (e) {
      console.error('保存配置失败:', e)
    }
  }

  async function selectFolders(): Promise<string[]> {
    try {
      return await invoke<string[]>('select_folders')
    } catch (e) {
      console.error('选择文件夹失败:', e)
      return []
    }
  }

  // 添加项目（支持多选）
  async function addProjects() {
    const folders = await selectFolders()
    if (folders.length === 0) return
    let added = 0
    for (const folder of folders) {
      if (config.value.projects.some(p => p.path === folder)) continue
      try {
        const project = await invoke<Project>('add_project', { path: folder })
        const customName = config.value.custom_names[project.path]
        if (customName) project.name = customName
        if (config.value.favorites.includes(project.id)) project.is_favorite = true
        config.value.projects.push(project)
        // Assign next sort_order
        const maxOrder = Math.max(0, ...config.value.projects.map(p => p.sort_order))
        project.sort_order = maxOrder + 1
        fetchSingleGitInfo(project)
        added++
      } catch (e) {
        console.error('添加项目失败:', folder, e)
      }
    }
    if (added > 0) await saveConfig()
    return added
  }

  // 批量扫描文件夹（支持多选）
  async function addWorkspaceFolders() {
    const folders = await selectFolders()
    if (folders.length === 0) return
    for (const folder of folders) {
      if (!config.value.workspace_folders.includes(folder)) {
        config.value.workspace_folders.push(folder)
      }
    }
    await scanAllProjects()
  }

  async function removeWorkspaceFolder(folder: string) {
    config.value.workspace_folders = config.value.workspace_folders.filter(f => f !== folder)
    config.value.projects = config.value.projects.filter(p => !p.path.startsWith(folder))
    await saveConfig()
  }

  async function scanAllProjects() {
    loading.value = true
    try {
      const allProjects: Project[] = []
      for (const folder of config.value.workspace_folders) {
        const projects = await invoke<Project[]>('scan_projects', { folder })
        allProjects.push(...projects)
      }
      const existingMap = new Map(config.value.projects.map(p => [p.path, p]))
      for (const proj of allProjects) {
        const existing = existingMap.get(proj.path)
        if (existing) {
          proj.is_favorite = existing.is_favorite
          proj.last_run_time = existing.last_run_time
          proj.last_build_time = existing.last_build_time
          proj.git_url = existing.git_url
          proj.branch = existing.branch
        }
        const customName = config.value.custom_names[proj.path]
        if (customName) proj.name = customName
        if (config.value.favorites.includes(proj.id)) proj.is_favorite = true
      }
      config.value.projects = allProjects
      await saveConfig()
      refreshGitInfo()
    } catch (e) {
      console.error('扫描项目失败:', e)
    } finally {
      loading.value = false
    }
  }

  // 刷新所有项目信息（不清空列表）
  async function refreshAllProjects() {
    loading.value = true
    try {
      // Re-read each project's package.json
      const updated: Project[] = []
      for (const proj of config.value.projects) {
        try {
          const fresh = await invoke<Project>('add_project', { path: proj.path })
          // Preserve user data
          fresh.is_favorite = proj.is_favorite
          fresh.last_run_time = proj.last_run_time
          fresh.last_build_time = proj.last_build_time
          fresh.custom_dev_command = proj.custom_dev_command
          fresh.custom_build_command = proj.custom_build_command
          const customName = config.value.custom_names[proj.path]
          if (customName) fresh.name = customName
          updated.push(fresh)
        } catch {
          // Project no longer exists, remove it
        }
      }
      // Also scan workspace folders for new projects
      for (const folder of config.value.workspace_folders) {
        try {
          const scanned = await invoke<Project[]>('scan_projects', { folder })
          for (const proj of scanned) {
            if (!updated.some(p => p.path === proj.path)) {
              const customName = config.value.custom_names[proj.path]
              if (customName) proj.name = customName
              if (config.value.favorites.includes(proj.id)) proj.is_favorite = true
              updated.push(proj)
            }
          }
        } catch { /* ignore */ }
      }
      config.value.projects = updated
      await saveConfig()
      refreshGitInfo()
    } catch (e) {
      console.error('刷新失败:', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchSingleGitInfo(project: Project) {
    try {
      const gitUrl = await invoke<string>('get_remote_url', { path: project.path }).catch(() => '')
      const branch = await invoke<string>('get_branch', { path: project.path }).catch(() => '')
      console.log('[DevStation] git info:', project.dir_name, 'branch:', branch, 'url:', gitUrl)
      project.git_url = gitUrl
      project.branch = branch
      await saveConfig()
    } catch (e) {
      console.error('[DevStation] fetchGitInfo failed:', e)
    }
  }

  async function refreshGitInfo() {
    const promises = config.value.projects.map(async (proj) => {
      try {
        const [gitUrl, branch] = await Promise.all([
          invoke<string>('get_remote_url', { path: proj.path }),
          invoke<string>('get_branch', { path: proj.path }),
        ])
        proj.git_url = gitUrl
        proj.branch = branch
      } catch { /* ignore */ }
    })
    await Promise.allSettled(promises)
    await saveConfig()
  }

  // 批量删除
  async function removeProjects(ids: string[]) {
    if (ids.length === 0) return
    try {
      config.value = await invoke<AppConfig>('remove_projects', {
        config: config.value,
        ids,
      })
      selectedIds.value = []
    } catch (e) {
      console.error('删除项目失败:', e)
    }
  }

  async function runDev(projectPath: string) {
    const proj = config.value.projects.find(p => p.path === projectPath)
    if (!proj) return

    // If already running, stop it
    if (proj.port && runningPorts.value.has(proj.port)) {
      await stopDev(proj.port)
      return
    }

    try {
      await invoke('run_dev', {
        path: projectPath,
        packageManager: config.value.package_manager,
        devScript: config.value.dev_script,
        customCommand: proj.custom_dev_command || '',
      })
      proj.last_run_time = new Date().toISOString()
      await saveConfig()
      // After a short delay, detect if port becomes active
      if (proj.port) {
        setTimeout(() => checkPortRunning(proj.port!), 5000)
      }
    } catch (e) { console.error('运行 dev 失败:', e); throw e }
  }

  async function stopDev(port: number) {
    try {
      await invoke('stop_process_on_port', { port })
      runningPorts.value.delete(port)
    } catch (e) { console.error('停止失败:', e) }
  }

  async function checkPortRunning(port: number) {
    try {
      const inUse = await invoke<boolean>('detect_port_in_use', { port })
      if (inUse) runningPorts.value.add(port)
      else runningPorts.value.delete(port)
    } catch { /* ignore */ }
  }

  async function checkAllRunningPorts() {
    const ports = config.value.projects
      .map(p => p.port)
      .filter((p): p is number => p != null)
    for (const port of ports) {
      await checkPortRunning(port)
    }
  }

  async function runBuild(projectPath: string) {
    const proj = config.value.projects.find(p => p.path === projectPath)
    try {
      await invoke('run_build', {
        path: projectPath,
        packageManager: config.value.package_manager,
        buildScript: config.value.build_script,
        customCommand: proj?.custom_build_command || '',
      })
      if (proj) { proj.last_build_time = new Date().toISOString(); await saveConfig() }
    } catch (e) { console.error('打包失败:', e); throw e }
  }

  async function runScript(projectPath: string, script: string) {
    try {
      await invoke('run_script', { path: projectPath, script, packageManager: config.value.package_manager })
    } catch (e) { console.error('运行脚本失败:', e); throw e }
  }

  async function openInIde(projectPath: string) {
    try { await invoke('open_in_ide', { path: projectPath, ideCommand: config.value.ide_command }) }
    catch (e) { console.error('打开 IDE 失败:', e); throw e }
  }

  async function openInTerminal(projectPath: string) {
    try { await invoke('open_in_terminal', { path: projectPath }) }
    catch (e) { console.error('打开终端失败:', e); throw e }
  }

  async function openInFinder(projectPath: string) {
    try { await invoke('open_in_finder', { path: projectPath }) }
    catch (e) { console.error('打开 Finder 失败:', e); throw e }
  }

  async function toggleFavorite(projectId: string) {
    try {
      config.value = await invoke<AppConfig>('toggle_favorite', { config: config.value, projectId })
    } catch (e) { console.error('切换收藏失败:', e) }
  }

  async function updateProjectName(projectPath: string, name: string) {
    try {
      config.value = await invoke<AppConfig>('update_project_name', { config: config.value, path: projectPath, name })
    } catch (e) { console.error('更新项目名失败:', e) }
  }

  async function updateProjectCommand(projectPath: string, field: 'custom_dev_command' | 'custom_build_command', value: string) {
    const proj = config.value.projects.find(p => p.path === projectPath)
    if (proj) {
      proj[field] = value
      await saveConfig()
    }
  }

  async function checkOutdated(projectPath: string): Promise<OutdatedDep[]> {
    try {
      const deps = await invoke<OutdatedDep[]>('check_outdated', { path: projectPath })
      outdatedCache.value[projectPath] = deps
      return deps
    } catch (e) { console.error('检查过期依赖失败:', e); return [] }
  }

  async function batchPull(projectPaths: string[]) {
    try {
      return await invoke<Array<{ path: string; success: boolean; message: string }>>('batch_pull', { paths: projectPaths })
    } catch (e) { console.error('批量 pull 失败:', e); throw e }
  }

  function moveProject(projectId: string, direction: 'up' | 'down') {
    const idx = config.value.projects.findIndex(p => p.id === projectId)
    if (idx < 0) return
    const targetIdx = direction === 'up' ? idx - 1 : idx + 1
    if (targetIdx < 0 || targetIdx >= config.value.projects.length) return
    const list = config.value.projects
    ;[list[idx], list[targetIdx]] = [list[targetIdx], list[idx]]
    saveConfig()
  }

  async function updateSortOrder(projectPath: string, order: number) {
    const proj = config.value.projects.find(p => p.path === projectPath)
    if (proj) {
      proj.sort_order = order
      await saveConfig()
    }
  }

  const filteredProjects = computed(() => {
    let list = [...config.value.projects]

    if (filterMode.value === 'favorites') {
      list = list.filter(p => p.is_favorite)
    }

    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase()
      list = list.filter(p =>
        p.name.toLowerCase().includes(q) ||
        p.dir_name.toLowerCase().includes(q) ||
        p.path.toLowerCase().includes(q) ||
        p.framework.toLowerCase().includes(q)
      )
    }

    // Sort by sort_order (0 = unset, goes last), then by name
    list.sort((a, b) => {
      if (a.sort_order !== b.sort_order) return a.sort_order - b.sort_order
      return a.name.localeCompare(b.name, 'zh-CN')
    })

    return list
  })

  return {
    config, loading, searchQuery, viewMode, filterMode, currentView,
    outdatedCache, selectedIds, runningPorts,
    loadConfig, saveConfig, selectFolders,
    addProjects, addWorkspaceFolders, removeWorkspaceFolder,
    scanAllProjects, refreshAllProjects, removeProjects,
    runDev, stopDev, runBuild, runScript,
    checkPortRunning, checkAllRunningPorts,
    openInIde, openInTerminal, openInFinder,
    toggleFavorite, updateProjectName, updateProjectCommand, moveProject,
    updateSortOrder,
    checkOutdated, batchPull,
    filteredProjects,
  }
})
