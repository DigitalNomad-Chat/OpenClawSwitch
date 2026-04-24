<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { Loader2, Copy, Terminal } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import type { TmuxSession } from '@/types/remote'

interface Props {
  workspaceId: string | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  toast: [type: 'success' | 'error', message: string]
}>()

const sessions = ref<TmuxSession[]>([])
const loading = ref(false)

async function loadSessions() {
  if (!props.workspaceId) return
  loading.value = true
  try {
    sessions.value = await invoke<TmuxSession[]>('tmux_list_sessions')
  } catch (err: any) {
    const msg = String(err)
    if (msg.includes('SSH 未连接') || msg.includes('SSH 未认证')) {
      // 静默处理无 SSH 的情况
      sessions.value = []
    } else {
      emit('toast', 'error', `加载 tmux 会话失败: ${err}`)
      sessions.value = []
    }
  } finally {
    loading.value = false
  }
}

async function copyAttachCommand(sessionName: string) {
  if (!props.workspaceId) return
  try {
    const cmd = await invoke<string>('tmux_attach_session', {
      workspaceId: props.workspaceId,
      sessionName,
    })
    await navigator.clipboard.writeText(cmd)
    emit('toast', 'success', 'Attach 命令已复制')
  } catch (err) {
    emit('toast', 'error', `复制命令失败: ${err}`)
  }
}

async function openTerminalAttach(sessionName: string) {
  if (!props.workspaceId) return
  try {
    const cmd = await invoke<string>('tmux_attach_session', {
      workspaceId: props.workspaceId,
      sessionName,
    })
    await invoke('open_terminal_with_command', { command: cmd })
    emit('toast', 'success', '正在打开终端...')
  } catch (err) {
    emit('toast', 'error', `打开终端失败: ${err}`)
  }
}

onMounted(loadSessions)
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <h4 class="text-sm font-medium" style="color: var(--oc-text-primary);">tmux 会话</h4>
      <Button variant="outline" size="sm" :disabled="loading" @click="loadSessions">
        <Loader2 v-if="loading" class="w-4 h-4 animate-spin" />
        <span v-else>刷新</span>
      </Button>
    </div>

    <div v-if="!workspaceId" class="text-sm" style="color: var(--oc-text-muted);">
      未选择远程主机
    </div>

    <div v-else-if="sessions.length === 0 && !loading" class="text-sm" style="color: var(--oc-text-muted);">
      暂无 tmux 会话
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="session in sessions"
        :key="session.name"
        class="flex items-center justify-between gap-3 p-3 rounded-lg"
        style="background: var(--oc-card-elevated);"
      >
        <div class="min-w-0">
          <div class="text-sm font-medium truncate" style="color: var(--oc-text-primary);">
            {{ session.name }}
            <span
              v-if="session.attached"
              class="ml-2 text-xs px-1.5 py-0.5 rounded"
              style="background: var(--primary-100); color: var(--primary-700);"
            >
              attached
            </span>
          </div>
          <div class="text-xs mt-0.5" style="color: var(--oc-text-muted);">
            {{ session.windows }} 窗口
          </div>
        </div>

        <div class="flex items-center gap-2 flex-shrink-0">
          <Button variant="ghost" size="sm" @click="copyAttachCommand(session.name)">
            <Copy class="w-4 h-4" />
          </Button>
          <Button variant="outline" size="sm" @click="openTerminalAttach(session.name)">
            <Terminal class="w-4 h-4" />
            Attach
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
