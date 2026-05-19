<script setup lang="ts">
import {
  NLayout, NLayoutHeader, NLayoutContent, NButton, NSpace, NIcon,
  NCard, NList, NListItem, NInput, NForm, NFormItem,
  NRadio, NModal, NEmpty, useMessage
} from 'naive-ui'
import { ArrowBackOutline, AddOutline, CreateOutline, TrashOutline, FlashOutline } from '@vicons/ionicons5'
import { ref } from 'vue'
import { useAppStore } from '../stores/app'
import type { ModelProfile } from '../types'

const store = useAppStore()
const message = useMessage()

const showModal = ref(false)
const editingId = ref<string | null>(null)
const editForm = ref({ name: '', alias: '', token: '', base_url: '' })

const emit = defineEmits<{
  (e: 'back'): void
  (e: 'toggle-theme'): void
}>()

function openAdd() {
  editingId.value = null
  editForm.value = { name: '', alias: '', token: '', base_url: '' }
  showModal.value = true
}

function openEdit(profile: ModelProfile) {
  editingId.value = profile.id
  editForm.value = { name: profile.name, alias: profile.alias, token: profile.token, base_url: profile.base_url }
  showModal.value = true
}

function generateId() {
  return Date.now().toString(36) + Math.random().toString(36).slice(2)
}

async function handleSave() {
  const { name, alias, token, base_url } = editForm.value
  if (!name.trim()) { message.warning('请输入模型名称'); return }
  if (!alias.trim()) { message.warning('请输入别名'); return }
  if (!token.trim()) { message.warning('请输入 Token'); return }
  if (!base_url.trim()) { message.warning('请输入 Base URL'); return }

  if (editingId.value) {
    const profile = store.config.model_profiles.find(p => p.id === editingId.value)
    if (profile) {
      profile.name = name.trim()
      profile.alias = alias.trim()
      profile.token = token.trim()
      profile.base_url = base_url.trim()
    }
  } else {
    store.config.model_profiles.push({
      id: generateId(),
      name: name.trim(),
      alias: alias.trim(),
      token: token.trim(),
      base_url: base_url.trim(),
    })
  }
  await store.saveConfig()
  showModal.value = false
  message.success(editingId.value ? '已更新' : '已添加')
}

async function handleDelete(id: string) {
  store.config.model_profiles = store.config.model_profiles.filter(p => p.id !== id)
  if (store.config.active_model_profile_id === id) {
    store.config.active_model_profile_id = ''
  }
  await store.saveConfig()
  message.success('已删除')
}

async function handleActivate(profile: ModelProfile) {
  try {
    await store.activateModelProfile(profile)
    message.success(`已激活: ${profile.name}`)
  } catch (e: any) {
    message.error(`激活失败: ${e}`)
  }
}
</script>

<template>
  <NLayout class="models-layout" position="absolute">
    <NLayoutHeader bordered style="height: 56px; padding: 0 20px; display: flex; align-items: center; justify-content: space-between;">
      <NSpace align="center" :size="12">
        <NButton quaternary circle @click="emit('back')">
          <template #icon><NIcon :component="ArrowBackOutline" /></template>
        </NButton>
        <h2 style="font-size: 18px; font-weight: 600;">模型配置</h2>
      </NSpace>
      <NButton size="small" type="primary" @click="openAdd">
        <template #icon><NIcon :component="AddOutline" /></template>
        添加配置
      </NButton>
    </NLayoutHeader>

    <NLayoutContent style="flex: 1; overflow: auto; padding: 20px; max-width: 800px; margin: 0 auto; width: 100%;">
      <NCard title="Claude 模型切换">
        <NList v-if="store.config.model_profiles.length > 0" bordered>
          <NListItem v-for="profile in store.config.model_profiles" :key="profile.id">
            <div style="display: flex; align-items: center; justify-content: space-between; width: 100%;">
              <NSpace align="center" :size="12">
                <NRadio
                  :checked="store.config.active_model_profile_id === profile.id"
                  @update:checked="handleActivate(profile)"
                />
                <div>
                  <div style="font-weight: 600;">{{ profile.alias }} <span style="font-weight: 400; color: #999; font-size: 12px;">{{ profile.name }}</span></div>
                  <div style="font-size: 12px; color: #999; font-family: monospace;">{{ profile.base_url }}</div>
                </div>
              </NSpace>
              <NSpace :size="4">
                <NButton quaternary circle size="small" @click="openEdit(profile)">
                  <template #icon><NIcon :component="CreateOutline" /></template>
                </NButton>
                <NButton quaternary circle size="small" @click="handleActivate(profile)" type="primary">
                  <template #icon><NIcon :component="FlashOutline" /></template>
                </NButton>
                <NButton quaternary circle size="small" type="error" @click="handleDelete(profile.id)">
                  <template #icon><NIcon :component="TrashOutline" /></template>
                </NButton>
              </NSpace>
            </div>
          </NListItem>
        </NList>
        <NEmpty v-else description="暂无模型配置，点击右上角添加" />
      </NCard>
    </NLayoutContent>

    <NModal v-model:show="showModal">
      <NCard :title="editingId ? '编辑配置' : '添加配置'" style="width: 480px;">
        <NForm label-placement="left" label-width="90">
          <NFormItem label="模型名称">
            <NInput v-model:value="editForm.name" placeholder="如: glm-5.1" />
          </NFormItem>
          <NFormItem label="别名">
            <NInput v-model:value="editForm.alias" placeholder="如: 百度 glm-5" />
          </NFormItem>
          <NFormItem label="Token">
            <NInput v-model:value="editForm.token" type="password" show-password-on="click" placeholder="ANTHROPIC_AUTH_TOKEN" />
          </NFormItem>
          <NFormItem label="Base URL">
            <NInput v-model:value="editForm.base_url" placeholder="https://..." />
          </NFormItem>
        </NForm>
        <template #footer>
          <NSpace justify="end">
            <NButton @click="showModal = false">取消</NButton>
            <NButton type="primary" @click="handleSave">保存</NButton>
          </NSpace>
        </template>
      </NCard>
    </NModal>
  </NLayout>
</template>

<style scoped>
.models-layout {
  height: 100vh;
}
</style>
