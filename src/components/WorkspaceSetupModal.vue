<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import Button from './ui/Button.vue'
import Input from './ui/Input.vue'
import Label from './ui/Label.vue'
import Card from './ui/Card.vue'
import { Server, FolderOpen, HardDrive, Wifi, X, Loader2 } from 'lucide-vue-next'
import type { SshProfile } from '@/types/config'
import type { Workspace, AccessTier, SshfsMountInfo } from '@/types/workspace'

const props = defineProps<{
  profile?: SshProfile
}>()

const emit = defineEmits<{
  close: []
  saved: [workspace: Workspace]
}>()

const hasProfile = computed(() => !!props.profile)

const name = ref(props.profile?.name || props.profile ? `${props.profile!.username}@${props.profile!.host}` : '')
const accessTier = ref<AccessTier>(hasProfile.value ? 'ssh_channel' : 'manual_mount')
const mountPath = ref('')
const remoteConfigPath = ref('~/.openclaw/openclaw.json')
const saving = ref(false)
const error = ref('')

const detectedMounts = ref<SshfsMountInfo[]>([])
const detectingMounts = ref(false)
const detectError = ref('')

async function detectSshfsMounts() {
  detectingMounts.value = true
  detectError.value = ''
  try {
    const mounts = await invoke<SshfsMountInfo[]>('detect_sshfs_mounts')
    detectedMounts.value = mounts
  } catch (e) {
    detectError.value = String(e)
    detectedMounts.value = []
  } finally {
    detectingMounts.value = false
  }
}

function selectDetectedMount(mount: SshfsMountInfo) {
  mountPath.value = mount.mountPoint
}

async function browseMountPath() {
  const selected = await open({ directory: true })
  if (selected && typeof selected === 'string') {
    mountPath.value = selected
  }
}

watch(accessTier, (tier) => {
  if (tier === 'manual_mount') {
    if (!mountPath.value) {
      mountPath.value = props.profile ? `~/sshfs-${props.profile.host}` : '~/openclaw-mount'
    }
    detectSshfsMounts()
  }
  if (tier === 'auto_mount' && !mountPath.value) {
    mountPath.value = props.profile ? `~/auto-${props.profile.host}` : '~/auto-mount'
  }
})

const handleSave = async () => {
  if (!name.value.trim()) {
    error.value = '请输入 Workspace 名称'
    return
  }

  if ((accessTier.value === 'manual_mount' || accessTier.value === 'auto_mount') && !mountPath.value.trim()) {
    error.value = '请输入挂载路径'
    return
  }

  if (accessTier.value === 'ssh_channel' && !remoteConfigPath.value.trim()) {
    error.value = '请输入远程配置文件路径'
    return
  }

  saving.value = true
  error.value = ''

  const workspace: Workspace = {
    id: crypto.randomUUID(),
    name: name.value.trim(),
    accessTier: accessTier.value,
    sortOrder: 0,
  }

  if (props.profile) {
    workspace.sshProfileId = props.profile.id
  }

  if (accessTier.value === 'manual_mount' || accessTier.value === 'auto_mount') {
    workspace.mountPath = mountPath.value.trim()
  } else {
    workspace.remoteConfigPath = remoteConfigPath.value.trim()
  }

  emit('saved', workspace)
  saving.value = false
}
</script>

<template>
  <div class="oc-modal-overlay" @click.self="emit('close')">
    <Card class="oc-modal-card w-full max-w-md p-6">
      <div class="flex items-center justify-between mb-4">
        <h3 class="font-semibold text-lg flex items-center gap-2" style="color: var(--oc-text-primary);">
          <Server class="w-5 h-5" style="color: var(--oc-accent);" />
          保存远程 Workspace
        </h3>
        <Button variant="ghost" size="sm" @click="emit('close')" class="h-8 w-8 p-0">
          <X class="w-4 h-4" />
        </Button>
      </div>

      <p v-if="hasProfile" class="text-xs mb-4" style="color: var(--oc-text-muted);">
        连接成功！请为 <strong style="color: var(--oc-text-primary);">{{ profile?.username }}@{{ profile?.host }}:{{ profile?.port }}</strong> 配置 Workspace。
      </p>
      <p v-else class="text-xs mb-4" style="color: var(--oc-text-muted);">
        添加一个已有的本地挂载目录作为 Workspace。
      </p>

      <div class="space-y-4">
        <div>
          <Label class="mb-1 block text-xs">Workspace 名称</Label>
          <Input v-model="name" placeholder="例如：Mac Mini" />
        </div>

        <div>
          <Label class="mb-1 block text-xs">访问方式</Label>
          <div class="grid gap-2" :class="hasProfile ? 'grid-cols-3' : 'grid-cols-1'">
            <button
              v-if="hasProfile"
              type="button"
              class="rounded-lg border p-3 text-center transition-colors"
              :style="{
                borderColor: accessTier === 'ssh_channel' ? 'var(--oc-accent)' : 'var(--oc-card-border)',
                background: accessTier === 'ssh_channel' ? 'color-mix(in srgb, var(--oc-accent) 8%, transparent)' : 'var(--oc-card-elevated)'
              }"
              @click="accessTier = 'ssh_channel'"
            >
              <Wifi class="w-4 h-4 mx-auto mb-1" :style="{ color: accessTier === 'ssh_channel' ? 'var(--oc-accent)' : 'var(--oc-text-muted)' }" />
              <div class="text-xs font-medium" style="color: var(--oc-text-primary);">SSH 通道</div>
              <div class="text-[10px] mt-0.5" style="color: var(--oc-text-muted);">纯 SSH</div>
            </button>
            <button
              type="button"
              class="rounded-lg border p-3 text-center transition-colors"
              :style="{
                borderColor: accessTier === 'manual_mount' ? 'var(--oc-accent)' : 'var(--oc-card-border)',
                background: accessTier === 'manual_mount' ? 'color-mix(in srgb, var(--oc-accent) 8%, transparent)' : 'var(--oc-card-elevated)'
              }"
              @click="accessTier = 'manual_mount'"
            >
              <FolderOpen class="w-4 h-4 mx-auto mb-1" :style="{ color: accessTier === 'manual_mount' ? 'var(--oc-accent)' : 'var(--oc-text-muted)' }" />
              <div class="text-xs font-medium" style="color: var(--oc-text-primary);">手动挂载</div>
              <div class="text-[10px] mt-0.5" style="color: var(--oc-text-muted);">已有 sshfs</div>
            </button>
            <button
              v-if="hasProfile"
              type="button"
              class="rounded-lg border p-3 text-center transition-colors"
              :style="{
                borderColor: accessTier === 'auto_mount' ? 'var(--oc-accent)' : 'var(--oc-card-border)',
                background: accessTier === 'auto_mount' ? 'color-mix(in srgb, var(--oc-accent) 8%, transparent)' : 'var(--oc-card-elevated)'
              }"
              @click="accessTier = 'auto_mount'"
            >
              <HardDrive class="w-4 h-4 mx-auto mb-1" :style="{ color: accessTier === 'auto_mount' ? 'var(--oc-accent)' : 'var(--oc-text-muted)' }" />
              <div class="text-xs font-medium" style="color: var(--oc-text-primary);">自动挂载</div>
              <div class="text-[10px] mt-0.5" style="color: var(--oc-text-muted);">应用自动 sshfs</div>
            </button>
          </div>
        </div>

        <div v-if="accessTier === 'manual_mount' || accessTier === 'auto_mount'">
          <Label class="mb-1 block text-xs">本地挂载路径</Label>

          <!-- 自动检测卡片（仅 manual_mount） -->
          <template v-if="accessTier === 'manual_mount'">
            <div v-if="detectingMounts" class="flex items-center gap-2 py-2 text-[11px]" style="color: var(--oc-text-muted);">
              <Loader2 class="w-3.5 h-3.5 spin" />
              正在检测 sshfs 挂载点…
            </div>

            <div v-else-if="detectedMounts.length > 0" class="grid gap-2 mb-3">
              <div
                v-for="m in detectedMounts"
                :key="m.mountPoint"
                class="rounded-lg border p-2.5 cursor-pointer transition-colors"
                :style="{
                  borderColor: mountPath === m.mountPoint ? 'var(--oc-accent)' : 'var(--oc-card-border)',
                  background: mountPath === m.mountPoint ? 'color-mix(in srgb, var(--oc-accent) 8%, transparent)' : 'var(--oc-card-elevated)'
                }"
                @click="selectDetectedMount(m)"
              >
                <div class="text-xs font-medium truncate" style="color: var(--oc-text-primary);">
                  {{ m.mountPoint }}
                </div>
                <div v-if="m.remoteHost || m.remotePath" class="text-[10px] truncate mt-0.5" style="color: var(--oc-text-muted);">
                  远程来源：{{ m.remoteHost || '' }}:{{ m.remotePath || '' }}
                </div>
              </div>
            </div>

            <div v-else-if="!detectingMounts" class="mb-3 text-[11px]" style="color: var(--oc-text-muted);">
              未检测到 sshfs 挂载点，请手动输入路径或点击下方浏览按钮选择。
            </div>
          </template>

          <div class="flex items-center gap-2">
            <Input v-model="mountPath" placeholder="例如：~/macmini-openclaw" class="flex-1" />
            <Button variant="outline" size="sm" @click="browseMountPath">浏览…</Button>
          </div>
          <p class="mt-1 text-[10px]" style="color: var(--oc-text-muted);">
            {{ accessTier === 'manual_mount' ? '指向已挂载的本地目录' : '应用将自动挂载到该路径' }}
          </p>
        </div>

        <div v-else>
          <Label class="mb-1 block text-xs">远程配置文件路径</Label>
          <Input v-model="remoteConfigPath" placeholder="例如：~/.openclaw/openclaw.json" />
        </div>

        <div
          v-if="error"
          class="rounded-[10px] border p-2 text-sm"
          style="border-color: color-mix(in srgb, var(--oc-danger) 58%, transparent); color: var(--oc-danger); background: color-mix(in srgb, var(--oc-danger) 12%, transparent);"
        >
          {{ error }}
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 mt-5">
        <Button variant="ghost" @click="emit('close')">跳过</Button>
        <Button @click="handleSave" :disabled="saving">
          {{ saving ? '保存中...' : '保存 Workspace' }}
        </Button>
      </div>
    </Card>
  </div>
</template>

<style scoped>
.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
