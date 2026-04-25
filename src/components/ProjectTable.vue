<script setup lang="ts">
import { h, ref } from 'vue'
import {
  NDataTable, NCard, NSpace, NButton, NIcon, NTag, NTooltip, NDropdown,
  NEmpty, useMessage, NModal, NList, NListItem, NInput, NInputGroup, NForm, NFormItem
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import {
  PlayOutline, BuildOutline, OpenOutline,
  StarOutline, Star, EllipsisVerticalOutline, StopOutline, CopyOutline
} from '@vicons/ionicons5'
import { useAppStore } from '../stores/app'
import type { Project, ViewMode, OutdatedDep } from '../types'

defineProps<{
  projects: Project[]
  viewMode: ViewMode
}>()

const store = useAppStore()
const message = useMessage()
const editingName = ref<string | null>(null)
const editNameValue = ref('')
const showOutdatedModal = ref(false)
const outdatedDeps = ref<OutdatedDep[]>([])
const outdatedProject = ref('')
const showCommandModal = ref(false)
const commandProject = ref<Project | null>(null)
const editDevCommand = ref('')
const editBuildCommand = ref('')
const editingSortId = ref<string | null>(null)
const editSortValue = ref('')

async function handleRunDev(project: Project) {
  try {
    const result = await store.runDev(project.path)
    if (result === 'started') message.success(`正在启动 ${project.name}...`)
    else if (result === 'stopped') message.success(`已停止 ${project.name}`)
  } catch (e: any) {
    message.error(e as string)
  }
}

function getDevActionState(project: Project) {
  return store.devActionStates[project.path]
}

function isDevRunning(project: Project) {
  return project.port != null && store.runningPorts.has(project.port)
}

async function handleBuild(project: Project) {
  try {
    await store.runBuild(project.path)
    message.success(`正在打包 ${project.name}...`)
  } catch (e: any) {
    message.error(e as string)
  }
}

async function handleOpenIde(project: Project) {
  try {
    await store.openInIde(project.path)
  } catch (e: any) {
    message.error(e as string)
  }
}

async function handleOpenTerminal(project: Project) {
  try {
    await store.openInTerminal(project.path)
  } catch (e: any) {
    message.error(e as string)
  }
}

async function handleOpenFinder(project: Project) {
  try {
    await store.openInFinder(project.path)
  } catch (e: any) {
    message.error(e as string)
  }
}

function startEditName(project: Project) {
  editingName.value = project.path
  editNameValue.value = project.name
}

async function saveName(project: Project) {
  if (editNameValue.value.trim()) {
    await store.updateProjectName(project.path, editNameValue.value.trim())
    message.success('名称已更新')
  }
  editingName.value = null
}

async function handleCheckOutdated(project: Project) {
  outdatedProject.value = project.name
  outdatedDeps.value = await store.checkOutdated(project.path)
  showOutdatedModal.value = true
}

async function handlePull(project: Project) {
  try {
    await store.batchPull([project.path])
    message.success(`${project.name} pull 成功`)
  } catch {
    message.error(`${project.name} pull 失败`)
  }
}

function openCommandModal(project: Project) {
  commandProject.value = project
  editDevCommand.value = project.custom_dev_command || ''
  editBuildCommand.value = project.custom_build_command || ''
  showCommandModal.value = true
}

async function saveCommands() {
  if (!commandProject.value) return
  await store.updateProjectCommand(commandProject.value.path, 'custom_dev_command', editDevCommand.value)
  await store.updateProjectCommand(commandProject.value.path, 'custom_build_command', editBuildCommand.value)
  showCommandModal.value = false
  message.success('命令配置已保存')
}

async function handleRemoveSingle(project: Project) {
  await store.removeProjects([project.id])
  message.success(`已移除 ${project.name}`)
}

function handleCopyInfo(project: Project) {
  const text = `${project.name} ${project.dir_name} tag-bzh-v${project.version}`
  navigator.clipboard.writeText(text)
  message.success('已复制: ' + text)
}

function getMoreActions(project: Project) {
  return [
    { label: '打开终端', key: 'terminal' },
    { label: '在 Finder 中显示', key: 'finder' },
    { label: 'Git Pull', key: 'pull' },
    { label: '检查过期依赖', key: 'outdated' },
    { label: '配置命令', key: 'command' },
    { type: 'divider', key: 'd2' },
    { label: '从列表移除', key: 'remove', props: { style: 'color: #d03050;' } },
    { type: 'divider', key: 'd1' },
    ...Object.keys(project.scripts)
      .filter(s => !['dev', 'build', 'start'].includes(s))
      .map(s => ({ label: `运行: ${s}`, key: `script:${s}` })),
  ]
}

function handleMoreAction(key: string, project: Project) {
  if (key === 'terminal') handleOpenTerminal(project)
  else if (key === 'finder') handleOpenFinder(project)
  else if (key === 'pull') handlePull(project)
  else if (key === 'outdated') handleCheckOutdated(project)
  else if (key === 'command') openCommandModal(project)
  else if (key === 'remove') handleRemoveSingle(project)
  else if (key.startsWith('script:')) {
    const script = key.slice(7)
    store.runScript(project.path, script)
    message.success(`正在运行 ${script}...`)
  }
}

function getLastCommitMessage(project: Project) {
  return project.last_commit_message || '-'
}

function startEditSort(project: Project) {
  editingSortId.value = project.id
  editSortValue.value = project.sort_order ? String(project.sort_order) : ''
}

async function saveSort(project: Project) {
  const num = parseInt(editSortValue.value) || 0
  await store.updateSortOrder(project.path, num)
  editingSortId.value = null
}

const columns: DataTableColumns<Project> = [
  {
    type: 'selection',
  },
  {
    title: '',
    key: 'favorite',
    width: 40,
    render(row) {
      return h(
        NButton,
        { quaternary: true, circle: true, size: 'tiny', onClick: () => store.toggleFavorite(row.id) },
        { icon: () => h(NIcon, { size: 16, color: row.is_favorite ? '#f0a020' : '#ccc' }, () => h(row.is_favorite ? Star : StarOutline)) }
      )
    },
  },
  {
    title: '排序',
    key: 'sort_order',
    width: 60,
    render(row) {
      if (editingSortId.value === row.id) {
        return h(NInput, {
          value: editSortValue.value,
          'onUpdate:value': (v: string) => { editSortValue.value = v },
          size: 'small',
          style: 'width: 50px',
          autofocus: true,
          onBlur: () => saveSort(row),
          onKeyup: (e: KeyboardEvent) => { if (e.key === 'Enter') saveSort(row) },
        })
      }
      return h('span', {
        style: 'cursor: pointer; font-family: monospace; font-size: 13px; color: ' + (row.sort_order ? '#333' : '#ccc'),
        onDblclick: () => startEditSort(row),
        title: '双击编辑排序',
      }, row.sort_order || '-')
    },
  },
  {
    title: '项目',
    key: 'name',
    minWidth: 130,
    render(row) {
      if (editingName.value === row.path) {
        return h(NInputGroup, {}, () => [
          h(NInput, {
            value: editNameValue.value,
            'onUpdate:value': (v: string) => { editNameValue.value = v },
            size: 'small',
            onBlur: () => saveName(row),
            onKeyup: (e: KeyboardEvent) => { if (e.key === 'Enter') saveName(row) },
          }),
        ])
      }
      return h('div', { style: 'display: flex; flex-direction: column; gap: 2px;' }, [
        h('span', {
          style: 'font-weight: 500; cursor: pointer;',
          onDblclick: () => startEditName(row),
          title: '双击编辑名称',
        }, row.name),
        h('span', { style: 'font-size: 12px; color: #999;' }, row.dir_name),
      ])
    },
  },
  {
    title: '提交记录',
    key: 'last_commit_message',
    width: 220,
    render(row) {
      const message = getLastCommitMessage(row)
      return h(NTooltip, null, {
        trigger: () => h('span', {
          style: 'display: block; max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 13px;',
        }, message),
        default: () => message,
      })
    },
  },
  {
    title: '版本',
    key: 'version',
    minWidth: 180,
    render(row) {
      return h('span', { style: 'font-family: monospace; font-size: 13px;' }, `tag-bzh-v${row.version}`)
    },
  },
  {
    title: '分支',
    key: 'branch',
    minWidth: 120,
    render(row) {
      return h(NTag, { size: 'small', type: row.branch ? 'default' : 'info' }, () => row.branch || '-')
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 230,
    fixed: 'right',
    render(row) {
      return h(NSpace, { size: 4, align: 'center' }, () => [
        h(NTooltip, null, {
          trigger: () => {
            const isRunning = isDevRunning(row)
            const actionState = getDevActionState(row)
            return h(NButton, {
              size: 'small',
              type: isRunning ? 'error' : 'success',
              quaternary: true,
              loading: actionState != null,
              disabled: actionState != null,
              onClick: () => handleRunDev(row),
            }, {
              icon: () => h(NIcon, { component: isRunning ? StopOutline : PlayOutline }),
            })
          },
          default: () => {
            const actionState = getDevActionState(row)
            if (actionState === 'starting') return '启动中...'
            if (actionState === 'stopping') return '停止中...'
            const isRunning = isDevRunning(row)
            const cmd = row.custom_dev_command || `${store.config.package_manager} ${store.config.dev_script}`
            return isRunning ? `停止 (port :${row.port})` : cmd
          },
        }),
        h(NTooltip, null, {
          trigger: () => h(NButton, { size: 'small', type: 'warning', quaternary: true, onClick: () => handleBuild(row) },
            { icon: () => h(NIcon, { component: BuildOutline }) }),
          default: () => row.custom_build_command || `${store.config.package_manager} ${store.config.build_script}`,
        }),
        h(NTooltip, null, {
          trigger: () => h(NButton, { size: 'small', type: 'info', quaternary: true, onClick: () => handleOpenIde(row) },
            { icon: () => h(NIcon, { component: OpenOutline }) }),
          default: () => `用 ${store.config.ide_command} 打开`,
        }),
        h(NTooltip, null, {
          trigger: () => h(NButton, { size: 'small', quaternary: true, onClick: () => handleCopyInfo(row) },
            { icon: () => h(NIcon, { component: CopyOutline }) }),
          default: () => '复制信息',
        }),
        h(NDropdown, {
          options: getMoreActions(row),
          onSelect: (key: string) => handleMoreAction(key, row),
        }, {
          default: () => h(NButton, { size: 'small', quaternary: true },
            { icon: () => h(NIcon, { component: EllipsisVerticalOutline }) }),
        }),
      ])
    },
  },
]
</script>

<template>
  <div>
    <!-- Table View -->
    <NDataTable
      v-if="viewMode === 'table'"
      :columns="columns"
      :data="projects"
      :bordered="false"
      :row-key="(row: Project) => row.id"
      :checked-row-keys="store.selectedIds"
      @update:checked-row-keys="(keys: Array<string | number>) => store.selectedIds = keys as string[]"
      :scroll-x="1000"
      striped
      size="small"
    />

    <!-- Card View -->
    <div v-else class="card-grid">
      <NCard v-for="project in projects" :key="project.id" size="small" hoverable class="project-card">
        <div class="card-header">
          <div class="card-title">
            <NButton quaternary circle size="tiny" @click="store.toggleFavorite(project.id)">
              <template #icon>
                <NIcon :size="14" :color="project.is_favorite ? '#f0a020' : '#ccc'">
                  <component :is="project.is_favorite ? Star : StarOutline" />
                </NIcon>
              </template>
            </NButton>
            <span class="card-name">{{ project.name }}</span>
          </div>
          <span class="card-version">v{{ project.version }}</span>
        </div>
        <div class="card-commit-row">
          <NTooltip>
            <template #trigger>
              <span class="card-commit">{{ getLastCommitMessage(project) }}</span>
            </template>
            {{ getLastCommitMessage(project) }}
          </NTooltip>
        </div>
        <div class="card-info">
          <span class="card-dir">{{ project.dir_name }}</span>
          <NTag v-if="project.branch" size="tiny">{{ project.branch }}</NTag>
          <NTag v-if="project.port" size="tiny" type="info" :bordered="false">:{{ project.port }}</NTag>
        </div>
        <div class="card-actions">
          <template v-if="editingSortId === project.id">
            <NInput
              v-model:value="editSortValue"
              size="tiny"
              style="width: 50px"
              autofocus
              @blur="saveSort(project)"
              @keyup.enter="saveSort(project)"
            />
          </template>
          <template v-else>
            <NButton size="tiny" quaternary @dblclick="startEditSort(project)" title="双击编辑排序">
              <span style="font-family: monospace; font-size: 12px;">{{ project.sort_order || '-' }}</span>
            </NButton>
          </template>
          <NButton
            size="tiny"
            :type="isDevRunning(project) ? 'error' : 'success'"
            :loading="getDevActionState(project) != null"
            :disabled="getDevActionState(project) != null"
            @click="handleRunDev(project)"
          >
            <template #icon>
              <NIcon :component="isDevRunning(project) ? StopOutline : PlayOutline" />
            </template>
            {{ getDevActionState(project) === 'starting' ? '启动中' : getDevActionState(project) === 'stopping' ? '停止中' : isDevRunning(project) ? '停止' : '运行' }}
          </NButton>
          <NButton size="tiny" type="warning" @click="handleBuild(project)">
            <template #icon><NIcon :component="BuildOutline" /></template>
            打包
          </NButton>
          <NButton size="tiny" type="info" @click="handleOpenIde(project)">
            <template #icon><NIcon :component="OpenOutline" /></template>
            打开
          </NButton>
          <NTooltip>
            <template #trigger>
              <NButton size="tiny" quaternary @click="handleCopyInfo(project)">
                <template #icon><NIcon :component="CopyOutline" /></template>
              </NButton>
            </template>
            复制信息
          </NTooltip>
          <NDropdown :options="getMoreActions(project)" @select="(key: string) => handleMoreAction(key, project)">
            <NButton size="tiny" quaternary>
              <template #icon><NIcon :component="EllipsisVerticalOutline" /></template>
            </NButton>
          </NDropdown>
        </div>
      </NCard>
    </div>

    <NEmpty v-if="projects.length === 0 && store.config.workspace_folders.length > 0" description="没有匹配的项目" style="margin-top: 60px;" />

    <!-- Outdated deps modal -->
    <NModal v-model:show="showOutdatedModal" preset="card" title="过期依赖" style="max-width: 600px;">
      <template #header>
        <span>{{ outdatedProject }} - 过期依赖</span>
      </template>
      <NList v-if="outdatedDeps.length > 0" bordered>
        <NListItem v-for="dep in outdatedDeps" :key="dep.name">
          <div style="display: flex; align-items: center; justify-content: space-between; width: 100%;">
            <span style="font-weight: 500;">{{ dep.name }}</span>
            <NSpace :size="8" align="center">
              <NTag size="small" type="error">{{ dep.current }}</NTag>
              <span>→</span>
              <NTag size="small" type="success">{{ dep.latest }}</NTag>
              <NTag size="small" :bordered="false">{{ dep.dep_type }}</NTag>
            </NSpace>
          </div>
        </NListItem>
      </NList>
      <NEmpty v-else description="所有依赖都是最新的" />
    </NModal>

    <!-- Command config modal -->
    <NModal v-model:show="showCommandModal" preset="card" title="配置命令" style="max-width: 500px;">
      <template #header>
        <span>{{ commandProject?.name }} - 命令配置</span>
      </template>
      <NForm labelPlacement="left" labelWidth="80">
        <NFormItem label="Dev 命令">
          <NInput
            v-model:value="editDevCommand"
            placeholder="留空则使用全局默认"
            clearable
          />
        </NFormItem>
        <NFormItem label="Build 命令">
          <NInput
            v-model:value="editBuildCommand"
            placeholder="留空则使用全局默认"
            clearable
          />
        </NFormItem>
      </NForm>
      <div style="display: flex; justify-content: flex-end; gap: 8px; margin-top: 12px;">
        <NButton @click="showCommandModal = false">取消</NButton>
        <NButton type="primary" @click="saveCommands">保存</NButton>
      </div>
    </NModal>
  </div>
</template>

<style scoped>
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 12px;
}

.project-card {
  transition: transform 0.2s;
}

.project-card:hover {
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.card-name {
  font-weight: 600;
  font-size: 14px;
  flex-shrink: 0;
}

.card-commit-row {
  margin-bottom: 6px;
}

.card-commit {
  display: inline-block;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #666;
  font-size: 12px;
}

.card-version {
  font-family: monospace;
  font-size: 12px;
  color: #999;
  flex-shrink: 0;
}

.card-info {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 12px;
  color: #666;
}

.card-dir {
  font-family: monospace;
}

.card-actions {
  display: flex;
  gap: 6px;
}
</style>
