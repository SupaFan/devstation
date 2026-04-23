<script setup lang="ts">
import {
  NLayout, NLayoutHeader, NLayoutContent, NButton, NSpace, NInput, NSelect,
  NIcon, NTooltip, NSpin, NEmpty, NTag, NPopconfirm, useMessage
} from 'naive-ui'
import {
  SettingsOutline, RefreshOutline, AddOutline, SearchOutline,
  GridOutline, ListOutline, CloudDownloadOutline, FolderOpenOutline,
  TrashOutline
} from '@vicons/ionicons5'
import { useAppStore } from '../stores/app'
import ProjectTable from '../components/ProjectTable.vue'

const store = useAppStore()
const message = useMessage()

const filterOptions = [
  { label: '全部项目', value: 'all' },
  { label: '已收藏', value: 'favorites' },
]

const emit = defineEmits<{
  (e: 'toggle-theme'): void
}>()

async function handleAddProject() {
  try {
    const added = await store.addProjects()
    if (added && added > 0) message.success(`已添加 ${added} 个项目`)
  } catch (e: any) {
    message.error(e as string)
  }
}

async function handleAddFolder() {
  await store.addWorkspaceFolders()
}

async function handleRefresh() {
  await store.refreshAllProjects()
  message.success('刷新完成')
}

async function handleBatchPull() {
  const projects = store.config.projects
  if (projects.length === 0) {
    message.info('没有可拉取的项目')
    return
  }
  try {
    const results = await store.batchPull(projects.map(p => p.path))
    const failed = results.filter(r => !r.success)
    if (failed.length === 0) {
      message.success(`成功拉取 ${results.length} 个项目`)
    } else {
      message.warning(`${results.length - failed.length} 个成功, ${failed.length} 个失败`)
    }
  } catch {
    message.error('批量拉取失败')
  }
}

async function handleBatchDelete() {
  await store.removeProjects(store.selectedIds)
  message.success('已删除')
}
</script>

<template>
  <NLayout class="main-layout" position="absolute">
    <!-- Header -->
    <NLayoutHeader bordered style="height: 56px; padding: 0 20px; display: flex; align-items: center; justify-content: space-between;">
      <div class="header-left">
        <h2 class="logo-text">DevStation</h2>
        <NTag v-if="store.config.projects.length > 0" size="small" type="info" round>
          {{ store.config.projects.length }} 个项目
        </NTag>
      </div>
      <NSpace align="center" :size="8">
          <NButton size="small" type="primary" @click="handleAddProject">
            <template #icon><NIcon :component="AddOutline" /></template>
            添加项目
          </NButton>
        <NTooltip>
          <template #trigger>
            <NButton quaternary circle @click="handleAddFolder">
              <template #icon><NIcon :component="FolderOpenOutline" /></template>
            </NButton>
          </template>
          扫描文件夹
        </NTooltip>
        <NTooltip>
          <template #trigger>
            <NButton quaternary circle @click="handleBatchPull">
              <template #icon><NIcon :component="CloudDownloadOutline" /></template>
            </NButton>
          </template>
          批量 Git Pull
        </NTooltip>
        <NTooltip>
          <template #trigger>
            <NButton quaternary circle @click="handleRefresh" :loading="store.loading">
              <template #icon><NIcon :component="RefreshOutline" /></template>
            </NButton>
          </template>
          刷新
        </NTooltip>
        <NTooltip>
          <template #trigger>
            <NButton quaternary circle @click="emit('toggle-theme')">
              <template #icon><span style="font-size: 16px;">🌙</span></template>
            </NButton>
          </template>
          切换主题
        </NTooltip>
        <NTooltip>
          <template #trigger>
            <NButton quaternary circle @click="store.currentView = 'settings'">
              <template #icon><NIcon :component="SettingsOutline" /></template>
            </NButton>
          </template>
          设置
        </NTooltip>
      </NSpace>
    </NLayoutHeader>

    <!-- Toolbar -->
    <div class="toolbar">
      <NInput
        v-model:value="store.searchQuery"
        placeholder="搜索项目..."
        clearable
        style="max-width: 300px;"
      >
        <template #prefix><NIcon :component="SearchOutline" /></template>
      </NInput>

      <NSelect
        v-model:value="store.filterMode"
        :options="filterOptions"
        style="width: 120px;"
        size="small"
      />

      <div style="flex: 1;" />

      <!-- Batch actions -->
      <template v-if="store.selectedIds.length > 0">
        <NTag size="small" type="warning">已选 {{ store.selectedIds.length }} 项</NTag>
        <NPopconfirm @positive-click="handleBatchDelete">
          <template #trigger>
            <NButton size="small" type="error">
              <template #icon><NIcon :component="TrashOutline" /></template>
              删除选中
            </NButton>
          </template>
          确定删除选中的 {{ store.selectedIds.length }} 个项目？
        </NPopconfirm>
      </template>

      <NSpace :size="4">
        <NButton
          :type="store.viewMode === 'table' ? 'primary' : 'default'"
          size="small" quaternary
          @click="store.viewMode = 'table'"
        >
          <template #icon><NIcon :component="ListOutline" /></template>
          表格
        </NButton>
        <NButton
          :type="store.viewMode === 'card' ? 'primary' : 'default'"
          size="small" quaternary
          @click="store.viewMode = 'card'"
        >
          <template #icon><NIcon :component="GridOutline" /></template>
          卡片
        </NButton>
      </NSpace>
    </div>

    <!-- Content -->
    <NLayoutContent style="flex: 1; overflow: auto; padding: 16px 20px;">
      <NSpin :show="store.loading">
        <ProjectTable
          v-if="store.config.projects.length > 0"
          :projects="store.filteredProjects"
          :view-mode="store.viewMode"
        />
        <NEmpty v-else description="点击右上角「添加项目」选择项目目录" style="margin-top: 100px;" />
      </NSpin>
    </NLayoutContent>
  </NLayout>
</template>

<style scoped>
.main-layout {
  height: 100vh;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-text {
  font-size: 18px;
  font-weight: 600;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.toolbar {
  padding: 12px 20px;
  display: flex;
  align-items: center;
  gap: 12px;
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color);
}
</style>
