<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { Loader2, HardDrive, Unplug, AlertTriangle, Copy, ExternalLink, RefreshCw } from 'lucide-vue-next'
import { open as openExternal } from '@tauri-apps/api/shell'
import Button from '@/components/ui/Button.vue'
import type { SshfsMount } from '@/types/remote'

interface Props {
  workspaceId: string | null
  mountPath?: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  toast: [type: 'success' | 'error', message: string]
}>()

const mounts = ref<SshfsMount[]>([])
const macfuseDetected = ref<boolean | null>(null)
const loadingMounts = ref(false)
const mounting = ref(false)
const unmounting = ref(false)

async function loadMounts() {
  loadingMounts.value = true
  try {
    mounts.value = await invoke<SshfsMount[]>('sshfs_list_mounts')
    macfuseDetected.value = await invoke<boolean>('sshfs_detect_macfuse')
  } catch (err) {
    emit('toast', 'error', `加载挂载信息失败: ${err}`)
  } finally {
    loadingMounts.value = false
  }
}

async function handleMount() {
  if (!props.workspaceId) return
  mounting.value = true
  try {
    const msg = await invoke<string>('sshfs_mount', { workspaceId: props.workspaceId })
    emit('toast', 'success', msg)
    await loadMounts()
  } catch (err) {
    emit('toast', 'error', `挂载失败: ${err}`)
  } finally {
    mounting.value = false
  }
}

async function handleUnmount(localPath: string) {
  unmounting.value = true
  try {
    const msg = await invoke<string>('sshfs_unmount', { mountPath: localPath })
    emit('toast', 'success', msg)
    await loadMounts()
  } catch (err) {
    emit('toast', 'error', `卸载失败: ${err}`)
  } finally {
    unmounting.value = false
  }
}

async function copyCmd(cmd: string) {
  try {
    await navigator.clipboard.writeText(cmd)
    emit('toast', 'success', '命令已复制到剪贴板')
  } catch {
    emit('toast', 'error', '复制失败')
  }
}

async function openMacfuseSite() {
  try {
    await openExternal('https://macfuse.github.io/')
  } catch (err) {
    emit('toast', 'error', `打开链接失败: ${err}`)
  }
}

const isMounted = computed(() => {
  if (!props.mountPath) return false
  return mounts.value.some(m => m.localPath === props.mountPath || m.localPath.startsWith(props.mountPath!))
})

onMounted(loadMounts)
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <h4 class="text-sm font-medium" style="color: var(--oc-text-primary);">SSHFS 挂载</h4>
      <Button variant="outline" size="sm" :disabled="loadingMounts" @click="loadMounts">
        <Loader2 v-if="loadingMounts" class="w-4 h-4 animate-spin" />
        <span v-else>刷新</span>
      </Button>
    </div>

    <!-- macFUSE 安装向导 -->
    <div
      v-if="macfuseDetected === false"
      class="oc-panel p-4 space-y-4"
    >
      <div class="flex items-center gap-2">
        <AlertTriangle class="w-4 h-4" style="color: var(--warning);" />
        <span class="text-sm font-medium" style="color: var(--oc-text-primary);">macFUSE 安装向导</span>
      </div>

      <div class="space-y-4">
        <!-- 步骤 1 -->
        <div class="flex gap-3">
          <div
            class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium flex-shrink-0"
            style="background: var(--oc-accent-soft); color: var(--oc-accent);"
          >1</div>
          <div class="flex-1 min-w-0">
            <div class="text-sm" style="color: var(--oc-text-primary);">卸载 Homebrew 版本（若安装过）</div>
            <div class="text-xs mt-0.5" style="color: var(--oc-text-muted);">brew 安装的 macFUSE 与官方 PKG 冲突，建议先卸载</div>
            <div class="flex items-center gap-2 mt-1.5">
              <code class="text-xs px-2 py-1 rounded" style="background: var(--oc-bg-secondary); color: var(--oc-text-primary);">brew uninstall --cask macfuse</code>
              <Button variant="ghost" size="sm" @click="copyCmd('brew uninstall --cask macfuse')">
                <Copy class="w-3.5 h-3.5" />
              </Button>
            </div>
          </div>
        </div>

        <!-- 步骤 2 -->
        <div class="flex gap-3">
          <div
            class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium flex-shrink-0"
            style="background: var(--oc-accent-soft); color: var(--oc-accent);"
          >2</div>
          <div class="flex-1 min-w-0">
            <div class="text-sm" style="color: var(--oc-text-primary);">下载官方 macFUSE PKG</div>
            <div class="text-xs mt-0.5" style="color: var(--oc-text-muted);">Homebrew 版本在 macOS 上存在兼容性问题，请从官网下载</div>
            <Button variant="outline" size="sm" class="mt-1.5" @click="openMacfuseSite">
              <ExternalLink class="w-3.5 h-3.5 mr-1" />
              打开 macfuse.github.io
            </Button>
          </div>
        </div>

        <!-- 步骤 3 -->
        <div class="flex gap-3">
          <div
            class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium flex-shrink-0"
            style="background: var(--oc-accent-soft); color: var(--oc-accent);"
          >3</div>
          <div class="flex-1 min-w-0">
            <div class="text-sm" style="color: var(--oc-text-primary);">验证安装</div>
            <div class="text-xs mt-0.5" style="color: var(--oc-text-muted);">安装完成后在终端运行以下命令检查版本</div>
            <div class="flex items-center gap-2 mt-1.5">
              <code class="text-xs px-2 py-1 rounded" style="background: var(--oc-bg-secondary); color: var(--oc-text-primary);">sshfs --version</code>
              <Button variant="ghost" size="sm" @click="copyCmd('sshfs --version')">
                <Copy class="w-3.5 h-3.5" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      <!-- 特别提示 -->
      <div
        class="text-xs p-3 rounded-lg"
        style="background: var(--oc-bg-secondary); color: var(--oc-text-muted);"
      >
        macOS Sequoia 用户：新版 macFUSE 支持无内核扩展模式，安装后通常无需进入恢复模式启用扩展。若系统提示，请按指引在「系统设置 → 隐私与安全性」中允许。
      </div>

      <Button variant="outline" size="sm" :disabled="loadingMounts" @click="loadMounts">
        <RefreshCw class="w-4 h-4 mr-1" />
        我已安装，重新检测
      </Button>
    </div>

    <div v-if="mounts.length === 0 && !loadingMounts" class="text-sm" style="color: var(--oc-text-muted);">
      当前没有 SSHFS 挂载
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="mount in mounts"
        :key="mount.localPath"
        class="flex items-center justify-between gap-3 p-3 rounded-lg"
        style="background: var(--oc-card-elevated);"
      >
        <div class="min-w-0 flex items-center gap-3">
          <HardDrive class="w-5 h-5 flex-shrink-0" style="color: var(--oc-accent);" />
          <div>
            <div class="text-sm font-medium truncate" style="color: var(--oc-text-primary);">
              {{ mount.localPath }}
            </div>
            <div class="text-xs" style="color: var(--oc-text-muted);">
              {{ mount.remoteTarget }}
            </div>
          </div>
        </div>
        <Button variant="ghost" size="sm" :disabled="unmounting" @click="handleUnmount(mount.localPath)">
          <Unplug class="w-4 h-4" />
        </Button>
      </div>
    </div>

    <div v-if="workspaceId && mountPath && !isMounted" class="pt-2">
      <Button variant="outline" size="sm" :disabled="mounting || macfuseDetected === false" @click="handleMount">
        <HardDrive class="w-4 h-4" />
        自动挂载
      </Button>
    </div>
  </div>
</template>
