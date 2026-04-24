import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import type { Workspace, WorkspaceConnectionStatus } from '@/types/workspace'

const workspaces = ref<Workspace[]>([])
const activeWorkspaceId = ref<string | null>(null)
const connectionStatus = ref<WorkspaceConnectionStatus>('disconnected')
const connectionError = ref<string | null>(null)

export function useWorkspaceStore() {
  const activeWorkspace = computed(() =>
    workspaces.value.find(w => w.id === activeWorkspaceId.value) ?? null
  )

  const isLocalMode = computed(() => activeWorkspaceId.value === null)

  async function loadWorkspaces(): Promise<void> {
    try {
      workspaces.value = await invoke<Workspace[]>('workspace_load_all')
      await cleanupStaleAutoMounts()
    } catch (err) {
      console.error('加载 Workspace 失败:', err)
      workspaces.value = []
    }
  }

  async function saveWorkspace(workspace: Workspace): Promise<void> {
    await invoke('workspace_save', { workspace })
    await loadWorkspaces()
  }

  async function deleteWorkspace(id: string): Promise<void> {
    await invoke('workspace_delete', { id })
    if (activeWorkspaceId.value === id) {
      activeWorkspaceId.value = null
      connectionStatus.value = 'disconnected'
    }
    await loadWorkspaces()
  }

  async function cleanupStaleAutoMounts(): Promise<void> {
    try {
      const mounts = await invoke<Array<{ localPath: string; remoteTarget: string; mounted: boolean }>>('sshfs_list_mounts')
      for (const mount of mounts) {
        const owningWs = workspaces.value.find(w =>
          w.accessTier === 'auto_mount' &&
          w.mountPath &&
          (mount.localPath === w.mountPath || mount.localPath.startsWith(w.mountPath.replace(/\/$/, '') + '/'))
        )
        if (owningWs && owningWs.id !== activeWorkspaceId.value) {
          try {
            await invoke('sshfs_unmount', { mountPath: mount.localPath })
          } catch (err) {
            console.error(`启动时清理残留挂载点失败 (${mount.localPath}):`, err)
          }
        }
      }
    } catch (err) {
      console.error('清理残留挂载点失败:', err)
    }
  }

  async function setActiveWorkspace(id: string | null): Promise<void> {
    const previousId = activeWorkspaceId.value
    const previousWs = previousId ? workspaces.value.find(w => w.id === previousId) : null

    // 切出时自动卸载上一个 auto_mount
    if (previousWs?.accessTier === 'auto_mount' && previousWs.mountPath) {
      try {
        await invoke('sshfs_unmount', { mountPath: previousWs.mountPath })
      } catch (err) {
        console.error('auto_mount 卸载失败:', err)
      }
    }

    await invoke('workspace_set_active', { id })
    activeWorkspaceId.value = id

    if (id === null) {
      connectionStatus.value = 'disconnected'
      connectionError.value = null
      return
    }

    // 切入时自动挂载新的 auto_mount
    const ws = workspaces.value.find(w => w.id === id)
    if (ws?.accessTier === 'auto_mount') {
      try {
        await invoke('sshfs_mount', { workspaceId: id })
      } catch (err) {
        console.error('auto_mount 挂载失败:', err)
        // 挂载失败不影响 Workspace 激活，但抛出错误让调用方可以提示用户
        throw new Error(`自动挂载失败: ${err}`)
      }
    }
  }

  function setConnectionStatus(status: WorkspaceConnectionStatus, error?: string) {
    connectionStatus.value = status
    connectionError.value = error ?? null
  }

  return {
    workspaces,
    activeWorkspaceId,
    activeWorkspace,
    isLocalMode,
    connectionStatus,
    connectionError,
    loadWorkspaces,
    saveWorkspace,
    deleteWorkspace,
    setActiveWorkspace,
    setConnectionStatus,
    cleanupStaleAutoMounts,
  }
}
