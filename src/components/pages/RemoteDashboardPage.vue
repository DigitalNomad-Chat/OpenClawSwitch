<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  Server,
  Wifi,
  Globe,
  HardDrive,
  Loader2,
} from 'lucide-vue-next'
import RemoteHostCard from '@/components/remote/RemoteHostCard.vue'
import TmuxSessionList from '@/components/remote/TmuxSessionList.vue'
import SshfsMountPanel from '@/components/remote/SshfsMountPanel.vue'
import NetworkDiagnosticsPanel from '@/components/remote/NetworkDiagnosticsPanel.vue'
import { useWorkspaceStore } from '@/composables/useWorkspaceStore'
import { useWorkspaceConfig } from '@/composables/useWorkspaceConfig'
import type { TailscaleStatus, DiskUsage, KeylessAuthStatus } from '@/types/remote'
import type { EnvironmentStatus } from '@/types/config'

const props = defineProps<{
  showToast: (type: 'success' | 'error', message: string) => void
}>()

const workspaceStore = useWorkspaceStore()
const { checkEnvironment, healthCheck } = useWorkspaceConfig()

const activeTab = ref<'tmux' | 'sshfs' | 'network'>('tmux')
const envStatus = ref<EnvironmentStatus | null>(null)
const gatewayReachable = ref(false)
const tailscaleStatus = ref<TailscaleStatus | null>(null)
const diskUsage = ref<DiskUsage | null>(null)
const keylessAuth = ref<KeylessAuthStatus | null>(null)
const loading = ref(false)
const hasSshProfile = ref(false)

async function loadDashboardData() {
  if (!workspaceStore.activeWorkspaceId.value) return
  loading.value = true
  hasSshProfile.value = !!workspaceStore.activeWorkspace.value?.sshProfileId

  // 独立加载各模块，互不阻塞
  const results = await Promise.allSettled([
    checkEnvironment().then((r) => { envStatus.value = r }),
    healthCheck().then((r) => { gatewayReachable.value = r }),
    hasSshProfile.value
      ? invoke<TailscaleStatus>('tailscale_status').then((r) => { tailscaleStatus.value = r })
      : Promise.resolve(),
    hasSshProfile.value
      ? invoke<DiskUsage[]>('remote_disk_usage').then((usages) => { diskUsage.value = usages[0] || null })
      : Promise.resolve(),
    hasSshProfile.value
      ? invoke<KeylessAuthStatus>('ssh_check_keyless_auth').then((r) => { keylessAuth.value = r })
      : Promise.resolve(),
  ])

  results.forEach((res, idx) => {
    if (res.status === 'rejected') {
      console.error(`远程仪表盘数据加载失败 [${idx}]:`, res.reason)
    }
  })

  loading.value = false
}

async function handleDisconnect() {
  try {
    await invoke('ssh_disconnect')
    workspaceStore.setConnectionStatus('disconnected')
    props.showToast('success', '已断开连接')
  } catch (err) {
    props.showToast('error', `断开失败: ${err}`)
  }
}

async function handleReconnect() {
  // 重新连接逻辑：触发 SSH 连接流程
  // 实际由 App.vue 的 SshConnectModal 处理，这里仅更新状态提示
  props.showToast('success', '请通过顶部工具栏重新选择远程主机以连接')
}

async function handleSetupKeylessAuth() {
  const ws = workspaceStore.activeWorkspace.value
  if (!ws?.sshProfileId) {
    props.showToast('error', '当前 Workspace 未配置 SSH Profile')
    return
  }
  try {
    const profiles = await invoke<Array<{ id: string; host: string; port: number; username: string }>>('ssh_load_profiles')
    const profile = profiles.find(p => p.id === ws.sshProfileId)
    if (!profile) {
      props.showToast('error', '未找到 SSH Profile')
      return
    }
    const cmd = `ssh-copy-id -p ${profile.port} ${profile.username}@${profile.host}`
    await navigator.clipboard.writeText(cmd)
    props.showToast('success', `已复制命令：${cmd}，请在终端中粘贴执行`)
  } catch (err) {
    props.showToast('error', `获取 SSH Profile 失败: ${err}`)
  }
}

watch(
  () => workspaceStore.activeWorkspaceId.value,
  () => {
    if (workspaceStore.activeWorkspaceId.value) {
      loadDashboardData()
    }
  },
  { immediate: true }
)

onMounted(() => {
  if (workspaceStore.activeWorkspaceId.value) {
    loadDashboardData()
  }
})
</script>

<template>
  <div class="oc-remote-dashboard-page oc-page-root min-h-0 flex flex-col gap-3">
    <!-- 顶部 Host Status Bar -->
    <RemoteHostCard
      :workspace="workspaceStore.activeWorkspace.value"
      :connection-status="workspaceStore.connectionStatus.value"
      :env-status="envStatus"
      :keyless-auth="keylessAuth"
      @disconnect="handleDisconnect"
      @reconnect="handleReconnect"
      @setup-keyless-auth="handleSetupKeylessAuth"
    />

    <!-- 中部 2x2 状态卡 -->
    <div class="grid grid-cols-2 gap-3">
      <!-- OpenClaw 状态 -->
      <div class="oc-panel p-4 rounded-xl">
        <div class="flex items-center gap-2 mb-3">
          <Server class="w-4 h-4" style="color: var(--oc-accent);" />
          <span class="text-sm font-medium" style="color: var(--oc-text-primary);">OpenClaw</span>
        </div>
        <div v-if="loading" class="flex items-center gap-2 text-sm" style="color: var(--oc-text-muted);">
          <Loader2 class="w-4 h-4 animate-spin" />
          加载中...
        </div>
        <div v-else-if="envStatus" class="space-y-2">
          <div class="flex justify-between text-sm">
            <span style="color: var(--oc-text-muted);">安装状态</span>
            <span :style="{ color: envStatus.openclaw.installed ? 'var(--success)' : 'var(--oc-error)' }">
              {{ envStatus.openclaw.installed ? '已安装' : '未安装' }}
            </span>
          </div>
          <div class="flex justify-between text-sm">
            <span style="color: var(--oc-text-muted);">版本</span>
            <span style="color: var(--oc-text-primary);">{{ envStatus.openclaw.version || '--' }}</span>
          </div>
          <div class="flex justify-between text-sm">
            <span style="color: var(--oc-text-muted);">Node.js</span>
            <span style="color: var(--oc-text-primary);">
              {{ envStatus.node.installed ? envStatus.node.version : '未安装' }}
            </span>
          </div>
        </div>
        <div v-else class="text-sm" style="color: var(--oc-text-muted);">暂无数据</div>
      </div>

      <!-- Gateway 状态 -->
      <div class="oc-panel p-4 rounded-xl">
        <div class="flex items-center gap-2 mb-3">
          <Wifi class="w-4 h-4" style="color: var(--oc-accent);" />
          <span class="text-sm font-medium" style="color: var(--oc-text-primary);">网关</span>
        </div>
        <div v-if="loading" class="flex items-center gap-2 text-sm" style="color: var(--oc-text-muted);">
          <Loader2 class="w-4 h-4 animate-spin" />
          加载中...
        </div>
        <div v-else class="space-y-2">
          <div class="flex justify-between text-sm">
            <span style="color: var(--oc-text-muted);">健康状态</span>
            <span :style="{ color: gatewayReachable ? 'var(--success)' : 'var(--oc-error)' }">
              {{ gatewayReachable ? '可达' : '不可达' }}
            </span>
          </div>
        </div>
      </div>

      <!-- Tailscale 状态 -->
      <div class="oc-panel p-4 rounded-xl">
        <div class="flex items-center gap-2 mb-3">
          <Globe class="w-4 h-4" style="color: var(--oc-accent);" />
          <span class="text-sm font-medium" style="color: var(--oc-text-primary);">Tailscale</span>
        </div>
        <div v-if="loading" class="flex items-center gap-2 text-sm" style="color: var(--oc-text-muted);">
          <Loader2 class="w-4 h-4 animate-spin" />
          加载中...
        </div>
        <div v-else-if="tailscaleStatus" class="space-y-2">
          <div class="flex justify-between text-sm">
            <span style="color: var(--oc-text-muted);">运行状态</span>
            <span :style="{ color: tailscaleStatus.enabled ? 'var(--success)' : 'var(--oc-error)' }">
              {{ tailscaleStatus.enabled ? '运行中' : '未运行' }}
            </span>
          </div>
          <div class="flex justify-between text-sm">
            <span style="color: var(--oc-text-muted);">本机名称</span>
            <span style="color: var(--oc-text-primary);">{{ tailscaleStatus.selfName || '--' }}</span>
          </div>
          <div class="flex justify-between text-sm">
            <span style="color: var(--oc-text-muted);">在线节点</span>
            <span style="color: var(--oc-text-primary);">{{ tailscaleStatus.peers.filter(p => p.online).length }} / {{ tailscaleStatus.peers.length }}</span>
          </div>
        </div>
        <div v-else class="text-sm" style="color: var(--oc-text-muted);">
          {{ hasSshProfile ? '暂无数据' : '未配置 SSH，无法获取远程状态' }}
        </div>
      </div>

      <!-- 磁盘使用 -->
      <div class="oc-panel p-4 rounded-xl">
        <div class="flex items-center gap-2 mb-3">
          <HardDrive class="w-4 h-4" style="color: var(--oc-accent);" />
          <span class="text-sm font-medium" style="color: var(--oc-text-primary);">磁盘使用</span>
        </div>
        <div v-if="loading" class="flex items-center gap-2 text-sm" style="color: var(--oc-text-muted);">
          <Loader2 class="w-4 h-4 animate-spin" />
          加载中...
        </div>
        <div v-else-if="diskUsage" class="space-y-2">
          <div class="flex justify-between text-sm">
            <span style="color: var(--oc-text-muted);">总容量</span>
            <span style="color: var(--oc-text-primary);">{{ diskUsage.size }}</span>
          </div>
          <div class="flex justify-between text-sm">
            <span style="color: var(--oc-text-muted);">已用</span>
            <span style="color: var(--oc-text-primary);">{{ diskUsage.used }} ({{ diskUsage.capacity }})</span>
          </div>
          <div class="flex justify-between text-sm">
            <span style="color: var(--oc-text-muted);">可用</span>
            <span style="color: var(--oc-text-primary);">{{ diskUsage.available }}</span>
          </div>
        </div>
        <div v-else class="text-sm" style="color: var(--oc-text-muted);">
          {{ hasSshProfile ? '暂无数据' : '未配置 SSH，无法获取远程状态' }}
        </div>
      </div>
    </div>

    <!-- 底部标签页 -->
    <div class="oc-panel flex-1 min-h-0 flex flex-col rounded-xl overflow-hidden">
      <div class="flex items-center gap-1 border-b" style="border-color: var(--oc-divider);">
        <button
          class="px-4 py-2 text-sm font-medium transition-colors"
          :style="{
            color: activeTab === 'tmux' ? 'var(--oc-accent)' : 'var(--oc-text-muted)',
            borderBottom: activeTab === 'tmux' ? '2px solid var(--oc-accent)' : '2px solid transparent'
          }"
          @click="activeTab = 'tmux'"
        >
          tmux 会话
        </button>
        <button
          class="px-4 py-2 text-sm font-medium transition-colors"
          :style="{
            color: activeTab === 'sshfs' ? 'var(--oc-accent)' : 'var(--oc-text-muted)',
            borderBottom: activeTab === 'sshfs' ? '2px solid var(--oc-accent)' : '2px solid transparent'
          }"
          @click="activeTab = 'sshfs'"
        >
          SSHFS 挂载
        </button>
        <button
          class="px-4 py-2 text-sm font-medium transition-colors"
          :style="{
            color: activeTab === 'network' ? 'var(--oc-accent)' : 'var(--oc-text-muted)',
            borderBottom: activeTab === 'network' ? '2px solid var(--oc-accent)' : '2px solid transparent'
          }"
          @click="activeTab = 'network'"
        >
          网络诊断
        </button>
      </div>

      <div class="flex-1 overflow-auto p-4">
        <TmuxSessionList
          v-if="activeTab === 'tmux'"
          :workspace-id="workspaceStore.activeWorkspaceId.value"
          @toast="props.showToast"
        />
        <SshfsMountPanel
          v-else-if="activeTab === 'sshfs'"
          :workspace-id="workspaceStore.activeWorkspaceId.value"
          :mount-path="workspaceStore.activeWorkspace.value?.mountPath"
          @toast="props.showToast"
        />
        <NetworkDiagnosticsPanel
          v-else
          @toast="props.showToast"
        />
      </div>
    </div>
  </div>
</template>
