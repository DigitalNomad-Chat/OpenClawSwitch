<template>
  <div class="security-settings">
    <div class="settings-section">
      <div class="section-header">
        <div class="section-icon">
          <Shield :size="16" />
        </div>
        <div class="section-text">
          <h3 class="section-title">路径白名单</h3>
          <p class="section-desc">白名单内的路径允许 AI 读取文件时自动放行，无需逐次确认</p>
        </div>
      </div>

      <!-- 默认目录列表 -->
      <div class="path-list">
        <div
          v-for="path in defaultPaths"
          :key="path"
          class="path-item default"
        >
          <Folder :size="14" class="path-icon" />
          <span class="path-text">{{ path }}</span>
          <span class="path-badge">默认</span>
        </div>
      </div>

      <!-- 自定义目录列表 -->
      <div v-if="customPaths.length > 0" class="path-list custom-paths">
        <div
          v-for="(path, idx) in customPaths"
          :key="path"
          class="path-item"
        >
          <Folder :size="14" class="path-icon" />
          <span class="path-text">{{ path }}</span>
          <button class="path-remove-btn" @click="removePath(idx)" title="移除">
            <X :size="12" />
          </button>
        </div>
      </div>

      <!-- 添加按钮 -->
      <button class="add-path-btn" @click="addPath" :disabled="loading">
        <Plus :size="14" />
        <span>添加目录</span>
      </button>
    </div>

    <div class="settings-note">
      <Info :size="13" />
      <span>修改安全配置后需重启 AI 助手才能生效</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import { Shield, Folder, X, Plus, Info } from 'lucide-vue-next'

const emit = defineEmits<{
  showToast: [type: 'success' | 'error', message: string]
}>()

const props = defineProps<{
  showToast?: (type: 'success' | 'error', message: string) => void
}>()

const loading = ref(false)

const expandHome = (path: string): string => {
  // 前端展示用途，直接返回
  return path
}

const defaultPaths = ref<string[]>([
  expandHome('~/.openclaw'),
  expandHome('~/.claw'),
  '/tmp',
])
const customPaths = ref<string[]>([])

const loadConfig = async () => {
  loading.value = true
  try {
    const config = await invoke<{ allowed_paths: string[] }>('security_read_config')
    customPaths.value = config.allowed_paths || []
  } catch (err) {
    console.error('Failed to load security config:', err)
  } finally {
    loading.value = false
  }
}

const saveConfig = async (paths: string[]) => {
  try {
    await invoke('security_write_config', {
      config: { allowedPaths: paths },
    })
    if (props.showToast) {
      props.showToast('success', '安全配置已保存')
    }
  } catch (err) {
    console.error('Failed to save security config:', err)
    if (props.showToast) {
      props.showToast('error', '保存失败')
    }
  }
}

const addPath = async () => {
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择信任目录',
  })

  if (selected && typeof selected === 'string') {
    if (!customPaths.value.includes(selected) && !defaultPaths.value.includes(selected)) {
      customPaths.value.push(selected)
      await saveConfig(customPaths.value)
    }
  }
}

const removePath = async (idx: number) => {
  customPaths.value.splice(idx, 1)
  await saveConfig(customPaths.value)
}

onMounted(loadConfig)
</script>

<style scoped>
.security-settings {
  padding: 0.75rem 1.25rem;
}

.settings-section {
  margin-bottom: 0.75rem;
}

.section-header {
  display: flex;
  align-items: flex-start;
  gap: 0.625rem;
  margin-bottom: 0.75rem;
}

.section-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--primary-600) 12%, transparent);
  color: var(--primary-600);
  flex-shrink: 0;
  margin-top: 0.1rem;
}

.section-text {
  flex: 1;
}

.section-title {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--oc-text-primary);
}

.section-desc {
  margin: 0.2rem 0 0;
  font-size: 0.75rem;
  color: var(--oc-text-tertiary);
  line-height: 1.5;
}

/* Path list */
.path-list {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  margin-bottom: 0.5rem;
}

.path-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.625rem;
  border-radius: var(--radius-md);
  background: var(--bg-surface-elevated);
  border: 1px solid var(--oc-divider);
  font-size: 0.8125rem;
}

.path-item.default {
  opacity: 0.7;
}

.path-icon {
  color: var(--oc-text-tertiary);
  flex-shrink: 0;
}

.path-text {
  flex: 1;
  font-family: 'JetBrains Mono', 'Menlo', monospace;
  font-size: 0.75rem;
  color: var(--oc-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.path-badge {
  font-size: 0.6875rem;
  font-weight: 500;
  padding: 0.1rem 0.4rem;
  border-radius: 9999px;
  background: var(--bg-secondary);
  color: var(--oc-text-tertiary);
  flex-shrink: 0;
}

.path-remove-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  border-radius: var(--radius-sm);
  color: var(--oc-text-tertiary);
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.path-remove-btn:hover {
  background: color-mix(in srgb, var(--oc-danger) 12%, transparent);
  color: var(--oc-danger);
}

/* Add button */
.add-path-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.4rem 0.75rem;
  border: 1px dashed var(--oc-divider);
  border-radius: var(--radius-md);
  background: transparent;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--oc-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.add-path-btn:hover:not(:disabled) {
  border-color: var(--primary-600);
  color: var(--primary-600);
  background: color-mix(in srgb, var(--primary-600) 6%, transparent);
}

.add-path-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Note */
.settings-note {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.625rem;
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--oc-warning) 8%, transparent);
  font-size: 0.75rem;
  color: var(--oc-text-secondary);
}

.settings-note svg {
  color: var(--oc-warning);
  flex-shrink: 0;
}
</style>
