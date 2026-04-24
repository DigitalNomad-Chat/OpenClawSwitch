<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { Laptop, Server, Plus, Loader2, WifiOff, AlertCircle, Check, ChevronDown, Trash2 } from 'lucide-vue-next'
import type { Workspace } from '@/types/workspace'
import type { SshProfile } from '@/types/config'
import { useWorkspaceStore } from '@/composables/useWorkspaceStore'

const emit = defineEmits<{
  'workspace-changed': [workspaceId: string | null]
  'add-remote': []
  'add-local-mount': []
  'connect-ssh': [profileId: string]
}>()

const store = useWorkspaceStore()
const showDropdown = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

const currentLabel = computed(() => {
  if (store.isLocalMode.value) return '本地环境'
  return store.activeWorkspace.value?.name ?? '远程环境'
})

const currentIcon = computed(() => {
  if (store.isLocalMode.value) return Laptop
  return Server
})

function toggleDropdown() {
  showDropdown.value = !showDropdown.value
}

function closeDropdown() {
  showDropdown.value = false
}

function handleClickOutside(event: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    showDropdown.value = false
  }
}

async function selectLocal() {
  closeDropdown()
  try {
    await store.setActiveWorkspace(null)
    emit('workspace-changed', null)
  } catch (err) {
    console.error('切换本地环境失败:', err)
  }
}

async function selectWorkspace(ws: Workspace) {
  closeDropdown()
  if (store.activeWorkspaceId.value === ws.id) return
  try {
    await store.setActiveWorkspace(ws.id)
    emit('workspace-changed', ws.id)
    // manual_mount 无 SSH 配置时，不通知父组件建立 SSH 连接
    if (ws.sshProfileId) {
      emit('connect-ssh', ws.sshProfileId)
    }
  } catch (err) {
    console.error('切换 Workspace 失败:', err)
  }
}

function addRemote() {
  closeDropdown()
  emit('add-remote')
}

function addLocalMount() {
  closeDropdown()
  emit('add-local-mount')
}

async function removeWorkspace(e: Event, id: string) {
  e.stopPropagation()
  await store.deleteWorkspace(id)
}

onMounted(() => {
  store.loadWorkspaces()
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div ref="dropdownRef" class="workspace-switcher">
    <button class="workspace-trigger" @click.stop="toggleDropdown">
      <component :is="currentIcon" class="trigger-icon" />
      <span class="trigger-label">{{ currentLabel }}</span>
      <Loader2 v-if="store.connectionStatus.value === 'connecting'" class="trigger-status spin" />
      <Check v-else-if="!store.isLocalMode.value && store.connectionStatus.value === 'connected'" class="trigger-status connected" />
      <AlertCircle v-else-if="!store.isLocalMode.value && store.connectionStatus.value === 'error'" class="trigger-status error" />
      <ChevronDown class="trigger-chevron" :class="{ open: showDropdown }" />
    </button>

    <div v-if="showDropdown" class="workspace-dropdown">
      <div class="dropdown-section">配置源</div>

      <button
        class="dropdown-item"
        :class="{ active: store.isLocalMode.value }"
        @click="selectLocal"
      >
        <Laptop class="item-icon" />
        <span class="item-label">本地环境</span>
        <Check v-if="store.isLocalMode.value" class="item-check" />
      </button>

      <template v-if="store.workspaces.value.length > 0">
        <div class="dropdown-divider" />
        <div class="dropdown-section">远程主机</div>
        <button
          v-for="ws in store.workspaces.value"
          :key="ws.id"
          class="dropdown-item"
          :class="{ active: store.activeWorkspaceId.value === ws.id }"
          @click="selectWorkspace(ws)"
        >
          <Server class="item-icon" />
          <span class="item-label flex-1 truncate">{{ ws.name }}</span>
          <template v-if="ws.sshProfileId">
            <Loader2 v-if="store.activeWorkspaceId.value === ws.id && store.connectionStatus.value === 'connecting'" class="item-status spin" />
            <WifiOff v-else-if="store.activeWorkspaceId.value === ws.id && store.connectionStatus.value === 'disconnected'" class="item-status muted" />
            <AlertCircle v-else-if="store.activeWorkspaceId.value === ws.id && store.connectionStatus.value === 'error'" class="item-status error" />
            <Check v-else-if="store.activeWorkspaceId.value === ws.id && store.connectionStatus.value === 'connected'" class="item-status connected" />
          </template>
          <span
            class="item-delete"
            title="删除"
            role="button"
            tabindex="0"
            @click.stop="removeWorkspace($event, ws.id)"
            @keydown.enter.stop="removeWorkspace($event, ws.id)"
          >
            <Trash2 class="w-3 h-3" />
          </span>
        </button>
      </template>

      <div class="dropdown-divider" />
      <button class="dropdown-item add-item" @click="addRemote">
        <Plus class="item-icon" />
        <span class="item-label">添加 SSH 远程主机</span>
      </button>
      <button class="dropdown-item add-item" @click="addLocalMount">
        <Plus class="item-icon" />
        <span class="item-label">添加本地挂载点</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.workspace-switcher {
  position: relative;
  display: inline-flex;
}

.workspace-trigger {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 10px;
  border: 1px solid var(--oc-card-border);
  background: var(--oc-card-elevated);
  color: var(--oc-text-primary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.workspace-trigger:hover {
  border-color: var(--primary-300);
  background: var(--bg-primary);
}

.trigger-icon {
  width: 15px;
  height: 15px;
  flex-shrink: 0;
}

.trigger-label {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trigger-status {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.trigger-status.connected {
  color: var(--oc-success);
}

.trigger-status.error {
  color: var(--oc-danger);
}

.trigger-chevron {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  color: var(--oc-text-muted);
  transition: transform 0.2s ease;
}

.trigger-chevron.open {
  transform: rotate(180deg);
}

.workspace-dropdown {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  min-width: 220px;
  max-width: 280px;
  padding: 6px;
  border-radius: 12px;
  border: 1px solid var(--oc-card-border);
  background: var(--oc-card-elevated);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.12);
  z-index: 300;
}

.dropdown-section {
  padding: 6px 8px;
  font-size: 11px;
  font-weight: 600;
  color: var(--oc-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.dropdown-divider {
  height: 1px;
  margin: 4px 0;
  background: var(--oc-divider-soft);
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 8px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--oc-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.dropdown-item:hover {
  background: var(--oc-accent-soft);
  color: var(--oc-text-primary);
}

.dropdown-item.active {
  background: color-mix(in srgb, var(--oc-accent) 10%, transparent);
  color: var(--oc-accent);
}

.item-icon {
  width: 15px;
  height: 15px;
  flex-shrink: 0;
}

.item-label {
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-check {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  margin-left: auto;
  color: var(--oc-accent);
}

.item-status {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  margin-left: auto;
}

.item-status.connected {
  color: var(--oc-success);
}

.item-status.error {
  color: var(--oc-danger);
}

.item-status.muted {
  color: var(--oc-text-muted);
}

.item-delete {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  margin-left: 6px;
  padding: 0;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: var(--oc-text-muted);
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s ease, background 0.15s ease;
}

.dropdown-item:hover .item-delete {
  opacity: 1;
}

.item-delete:hover {
  background: color-mix(in srgb, var(--oc-danger) 12%, transparent);
  color: var(--oc-danger);
}

.add-item {
  color: var(--oc-accent);
  font-weight: 500;
}

.add-item:hover {
  background: color-mix(in srgb, var(--oc-accent) 10%, transparent);
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
