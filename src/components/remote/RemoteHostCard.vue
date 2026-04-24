<script setup lang="ts">
import { Monitor, Wifi, WifiOff, ShieldCheck, ShieldX, ShieldAlert } from 'lucide-vue-next'
import type { Workspace } from '@/types/workspace'
import type { EnvironmentStatus } from '@/types/config'
import type { KeylessAuthStatus } from '@/types/remote'

interface Props {
  workspace: Workspace | null
  connectionStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
  envStatus?: EnvironmentStatus | null
  keylessAuth?: KeylessAuthStatus | null
}

withDefaults(defineProps<Props>(), {
  envStatus: null,
  keylessAuth: null
})

const emit = defineEmits<{
  disconnect: []
  reconnect: []
  setupKeylessAuth: []
}>()

const statusMap: Record<string, { label: string; color: string; icon: any }> = {
  disconnected: { label: '未连接', color: 'var(--oc-text-muted)', icon: WifiOff },
  connecting: { label: '连接中', color: 'var(--warning)', icon: Wifi },
  connected: { label: '已连接', color: 'var(--success)', icon: Wifi },
  error: { label: '连接失败', color: 'var(--oc-error)', icon: WifiOff },
}

function authIcon(status: KeylessAuthStatus) {
  if (!status.localKeyExists) return ShieldAlert
  return status.keylessAuth ? ShieldCheck : ShieldX
}

function authColor(status: KeylessAuthStatus) {
  if (!status.localKeyExists) return 'var(--warning)'
  return status.keylessAuth ? 'var(--success)' : 'var(--oc-error)'
}
</script>

<template>
  <div class="oc-panel p-4 flex items-center justify-between gap-4">
    <div class="flex items-center gap-4">
      <div
        class="w-12 h-12 rounded-xl flex items-center justify-center"
        style="background: var(--oc-accent-soft);"
      >
        <Monitor class="w-6 h-6" style="color: var(--oc-accent);" />
      </div>
      <div>
        <h3 class="text-base font-semibold" style="color: var(--oc-text-primary);">
          {{ workspace?.name || '远程主机' }}
        </h3>
        <div class="flex items-center gap-2 mt-1">
          <component
            :is="statusMap[connectionStatus].icon"
            class="w-4 h-4"
            :style="{ color: statusMap[connectionStatus].color }"
          />
          <span class="text-sm" :style="{ color: statusMap[connectionStatus].color }">
            {{ statusMap[connectionStatus].label }}
          </span>
          <span v-if="envStatus" class="text-xs" style="color: var(--oc-text-muted);">
            · OpenClaw {{ envStatus.openclaw.version || '--' }}
          </span>
          <span v-if="envStatus" class="text-xs" style="color: var(--oc-text-muted);">
            · Node {{ envStatus.node.installed ? envStatus.node.version : '未安装' }}
          </span>
        </div>
        <!-- SSH 免密认证状态 -->
        <div v-if="connectionStatus === 'connected' && keylessAuth" class="flex items-center gap-1.5 mt-1">
          <component
            :is="authIcon(keylessAuth)"
            class="w-3.5 h-3.5"
            :style="{ color: authColor(keylessAuth) }"
          />
          <span class="text-xs" :style="{ color: authColor(keylessAuth) }">
            {{ keylessAuth.message }}
          </span>
          <button
            v-if="keylessAuth.localKeyExists && !keylessAuth.keylessAuth"
            class="text-xs ml-1 underline"
            style="color: var(--oc-accent);"
            @click="emit('setupKeylessAuth')"
          >
            一键配置
          </button>
        </div>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <button
        v-if="connectionStatus === 'connected'"
        class="oc-toolbar-btn h-9 px-3 text-sm"
        @click="emit('disconnect')"
      >
        断开连接
      </button>
      <button
        v-else
        class="oc-toolbar-btn h-9 px-3 text-sm"
        style="border-color: var(--oc-accent); color: var(--oc-accent);"
        @click="emit('reconnect')"
      >
        重新连接
      </button>
    </div>
  </div>
</template>
