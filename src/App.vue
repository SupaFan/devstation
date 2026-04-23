<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { NConfigProvider, NMessageProvider, NDialogProvider, darkTheme, zhCN } from 'naive-ui'
import { useAppStore } from './stores/app'
import MainView from './views/MainView.vue'
import SettingsView from './views/SettingsView.vue'

const store = useAppStore()
const isDark = ref(false)

onMounted(async () => {
  await store.loadConfig()
  if (store.config.workspace_folders.length > 0) {
    await store.scanAllProjects()
  }
  await store.checkAllRunningPorts()
})

function toggleTheme() {
  isDark.value = !isDark.value
}
</script>

<template>
  <NConfigProvider :theme="isDark ? darkTheme : undefined" :locale="zhCN">
    <NDialogProvider>
      <NMessageProvider>
        <div class="app-container">
          <MainView v-if="store.currentView === 'main'" :is-dark="isDark" @toggle-theme="toggleTheme" />
          <SettingsView v-else @back="store.currentView = 'main'" :is-dark="isDark" @toggle-theme="toggleTheme" />
        </div>
      </NMessageProvider>
    </NDialogProvider>
  </NConfigProvider>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  overflow: hidden;
}

.app-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #f5f7fa;
}

.app-container[data-theme="dark"] {
  background: #1a1a2e;
}
</style>
