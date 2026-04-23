<script setup lang="ts">
import {
  NLayout, NLayoutHeader, NLayoutContent, NButton, NSpace, NIcon,
  NCard, NList, NListItem, NInput, NSelect, NForm, NFormItem, useMessage
} from 'naive-ui'
import { ArrowBackOutline, TrashOutline, AddOutline } from '@vicons/ionicons5'
import { useAppStore } from '../stores/app'

const store = useAppStore()
const message = useMessage()

const pmOptions = [
  { label: 'pnpm', value: 'pnpm' },
  { label: 'npm', value: 'npm' },
  { label: 'yarn', value: 'yarn' },
]

const emit = defineEmits<{
  (e: 'back'): void
  (e: 'toggle-theme'): void
}>()

async function handleAddFolder() {
  await store.addWorkspaceFolders()
  message.success('已添加工作区文件夹')
}

function handleRemoveFolder(folder: string) {
  store.removeWorkspaceFolder(folder)
  message.success('已移除')
}

async function handleSave() {
  await store.saveConfig()
  message.success('设置已保存')
}
</script>

<template>
  <NLayout class="settings-layout" position="absolute">
    <NLayoutHeader bordered style="height: 56px; padding: 0 20px; display: flex; align-items: center; justify-content: space-between;">
      <NSpace align="center" :size="12">
        <NButton quaternary circle @click="emit('back')">
          <template #icon><NIcon :component="ArrowBackOutline" /></template>
        </NButton>
        <h2 style="font-size: 18px; font-weight: 600;">设置</h2>
      </NSpace>
      <NButton type="primary" @click="handleSave">保存设置</NButton>
    </NLayoutHeader>

    <NLayoutContent style="flex: 1; overflow: auto; padding: 20px; max-width: 800px; margin: 0 auto; width: 100%;">
      <!-- Workspace Folders -->
      <NCard title="工作区文件夹" style="margin-bottom: 16px;">
        <template #header-extra>
          <NButton size="small" @click="handleAddFolder">
            <template #icon><NIcon :component="AddOutline" /></template>
            添加
          </NButton>
        </template>
        <NList v-if="store.config.workspace_folders.length > 0" bordered>
          <NListItem v-for="folder in store.config.workspace_folders" :key="folder">
            <div style="display: flex; align-items: center; justify-content: space-between; width: 100%;">
              <span style="font-family: monospace; font-size: 13px;">{{ folder }}</span>
              <NButton quaternary circle size="small" type="error" @click="handleRemoveFolder(folder)">
                <template #icon><NIcon :component="TrashOutline" /></template>
              </NButton>
            </div>
          </NListItem>
        </NList>
        <div v-else style="color: #999; text-align: center; padding: 20px;">暂无工作区文件夹</div>
      </NCard>

      <!-- Tool Settings -->
      <NCard title="工具配置" style="margin-bottom: 16px;">
        <NForm label-placement="left" label-width="100">
          <NFormItem label="IDE 应用名">
            <NInput v-model:value="store.config.ide_command" placeholder="Trae" />
          </NFormItem>
          <NFormItem label="包管理器">
            <NSelect v-model:value="store.config.package_manager" :options="pmOptions" />
          </NFormItem>
          <NFormItem label="运行命令">
            <NInput v-model:value="store.config.dev_script" placeholder="dev" />
          </NFormItem>
          <NFormItem label="打包命令">
            <NInput v-model:value="store.config.build_script" placeholder="build" />
          </NFormItem>
        </NForm>
      </NCard>

      <!-- About -->
      <NCard title="关于">
        <NForm label-placement="left" label-width="100">
          <NFormItem label="应用名称">DevStation</NFormItem>
          <NFormItem label="版本">0.1.0</NFormItem>
          <NFormItem label="描述">前端多项目管理工作台</NFormItem>
        </NForm>
      </NCard>
    </NLayoutContent>
  </NLayout>
</template>

<style scoped>
.settings-layout {
  height: 100vh;
}
</style>
