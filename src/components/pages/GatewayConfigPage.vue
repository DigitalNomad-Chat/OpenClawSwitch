<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { RefreshCw, Save, AlertCircle, Shield, Network, Server, Ban, KeyRound, Plus, Trash2 } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Badge from '@/components/ui/Badge.vue'
import StyledSelect from '@/components/ui/StyledSelect.vue'
import HelpTooltip from '@/components/ui/HelpTooltip.vue'
import { useWorkspaceConfig } from '@/composables/useWorkspaceConfig'
import { useWorkspaceStore } from '@/composables/useWorkspaceStore'
import type { GatewayConfig, AuthConfig, OpenClawConfig } from '@/types/config'
import { createTagHandlers } from '@/composables/tagHandlers'

// ============================================================================
// Props
// ============================================================================

const props = defineProps<{
  showToast: (type: 'success' | 'error', message: string) => void
}>()

const workspaceStore = useWorkspaceStore()

// ============================================================================
// Composables
// ============================================================================

const { loading, saving, error, configSource, loadConfig, saveConfig } = useWorkspaceConfig()

// ============================================================================
// 响应式表单状态
// ============================================================================

const gatewayPort = ref(19633)
const gatewayMode = ref('local')
const gatewayBind = ref('loopback')
const authMode = ref('token')
const authToken = ref('')
const tailscaleMode = ref('off')
const hotReloadMode = ref('hybrid')
const denyCommands = ref<string[]>([])
const denyCommandsInput = ref('')
const isDirty = ref(false)

// Auth Profiles 管理
interface AuthProfileEntry {
  id: string
  provider: string
  mode: string
}
const authProfiles = ref<AuthProfileEntry[]>([])
const newProfileId = ref('')
const newProfileProvider = ref('')
const newProfileMode = ref('')

// 标签 handler（闭包捕获 ref）
const denyCommandsTag = createTagHandlers(denyCommands, denyCommandsInput)

// ============================================================================
// 计算属性 & 选项
// ============================================================================

const configPath = computed(() => configSource.value?.fileInfo.path ?? '')

const modeOptions = [
  { value: 'local', label: 'local', subtext: '本地运行' },
  { value: 'remote', label: 'remote', subtext: '远程模式' },
]

const bindOptions = [
  { value: 'loopback', label: 'loopback (127.0.0.1)', subtext: '仅本机访问' },
  { value: '0.0.0.0', label: '0.0.0.0', subtext: '所有网络接口（需注意安全）' },
]

const authModeOptions = [
  { value: 'token', label: 'token', subtext: 'Token 认证' },
  { value: 'none', label: 'none', subtext: '无认证（不推荐）' },
]

const tailscaleOptions = [
  { value: 'off', label: 'off', subtext: '关闭' },
  { value: 'serve', label: 'serve', subtext: '作为 Tailscale 服务' },
]

const hotReloadModeOptions = [
  { value: 'hybrid', label: 'hybrid', subtext: '安全改动即时生效，关键改动自动重启（推荐）' },
  { value: 'hot', label: 'hot', subtext: '仅安全改动即时生效，关键改动提示手动重启' },
  { value: 'restart', label: 'restart', subtext: '任何改动都自动重启 Gateway' },
  { value: 'off', label: 'off', subtext: '关闭监听，手动重启才生效' },
]

const authProfileModeOptions = [
  { value: 'token', label: 'token', subtext: 'Token 认证' },
  { value: 'oauth', label: 'oauth', subtext: 'OAuth 认证' },
  { value: 'api-key', label: 'api-key', subtext: 'API Key 认证' },
]

// ============================================================================
// 配置同步
// ============================================================================

function syncFormFromConfig() {
  if (!configSource.value) return
  const g = configSource.value.config.gateway ?? {}

  gatewayPort.value = g.port ?? 19633
  gatewayMode.value = g.mode ?? 'local'
  gatewayBind.value = g.bind ?? 'loopback'
  authMode.value = g.auth?.mode ?? 'token'
  authToken.value = g.auth?.token ?? ''
  tailscaleMode.value = g.tailscale?.mode ?? 'off'
  hotReloadMode.value = g.hotReloadMode ?? 'hybrid'
  denyCommands.value = [...(g.nodes?.denyCommands ?? [])]

  // Auth Profiles 同步
  const profiles = configSource.value.config.auth?.profiles
  if (profiles && typeof profiles === 'object') {
    authProfiles.value = Object.entries(profiles).map(([id, p]: [string, any]) => ({
      id,
      provider: p?.provider ?? '',
      mode: p?.mode ?? '',
    }))
  } else {
    authProfiles.value = []
  }

  isDirty.value = false
}

function buildGatewayConfig(): GatewayConfig {
  const config: GatewayConfig = {
    port: gatewayPort.value,
    mode: gatewayMode.value as 'local' | 'remote',
    bind: gatewayBind.value as 'loopback' | '0.0.0.0',
    auth: {
      mode: authMode.value as 'token' | 'none',
      ...(authMode.value === 'token' && authToken.value ? { token: authToken.value } : {}),
    },
    tailscale: {
      mode: tailscaleMode.value as 'serve' | 'off',
    },
  }

  if (denyCommands.value.length > 0) {
    config.nodes = { denyCommands: [...denyCommands.value] }
  }

  if (hotReloadMode.value !== 'hybrid') {
    config.hotReloadMode = hotReloadMode.value as 'hybrid' | 'hot' | 'restart' | 'off'
  }

  return config
}

/** 构建认证配置 */
function buildAuthConfig(): AuthConfig | undefined {
  if (authProfiles.value.length === 0) return undefined
  const profiles: Record<string, { provider?: string; mode?: string }> = {}
  for (const p of authProfiles.value) {
    if (p.id.trim()) {
      profiles[p.id.trim()] = {
        ...(p.provider ? { provider: p.provider } : {}),
        ...(p.mode ? { mode: p.mode } : {}),
      }
    }
  }
  return Object.keys(profiles).length > 0 ? { profiles } : undefined
}

/** 添加 Auth Profile */
function addAuthProfile() {
  const id = newProfileId.value.trim()
  if (!id) return
  if (authProfiles.value.some(p => p.id === id)) {
    props.showToast('error', `Profile "${id}" 已存在`)
    return
  }
  authProfiles.value.push({
    id,
    provider: newProfileProvider.value.trim(),
    mode: newProfileMode.value.trim(),
  })
  newProfileId.value = ''
  newProfileProvider.value = ''
  newProfileMode.value = ''
  isDirty.value = true
}

/** 删除 Auth Profile */
function removeAuthProfile(id: string) {
  authProfiles.value = authProfiles.value.filter(p => p.id !== id)
  isDirty.value = true
}

// ============================================================================
// 操作处理
// ============================================================================

async function handleRefresh() {
  await loadConfig()
  syncFormFromConfig()
  props.showToast('success', '已刷新')
}

async function handleSave() {
  try {
    if (!configSource.value) throw new Error('未加载配置')
    const auth = buildAuthConfig()
    const updated: OpenClawConfig = {
      ...configSource.value.config,
      gateway: buildGatewayConfig(),
      ...(auth ? { auth } : {}),
    }
    await saveConfig(updated)
    isDirty.value = false
    props.showToast('success', '配置已保存')
  } catch (err) {
    console.error('保存失败:', err)
    props.showToast('error', `保存失败: ${err}`)
  }
}

// ============================================================================
// 生命周期
// ============================================================================

onMounted(async () => {
  try {
    await loadConfig()
    syncFormFromConfig()
  } catch (err) {
    console.error('加载配置失败:', err)
  }
})

watch(
  () => workspaceStore.activeWorkspaceId.value,
  async () => {
    await loadConfig()
    syncFormFromConfig()
  }
)
</script>

<template>
  <div class="oc-gateway-page oc-page-root min-h-0 flex flex-col gap-3">
    <!-- 头部操作区 -->
    <section class="oc-panel flex-none p-4">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h3 style="font-size: var(--text-lg); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
            <Server class="w-5 h-5 inline-block mr-1 -mt-0.5" />
            网关配置
          </h3>
          <p class="mt-1" style="font-size: var(--text-sm); color: var(--oc-text-muted);">
            管理网关端口、绑定地址、认证方式和安全策略
          </p>
        </div>

        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" @click="handleRefresh" :disabled="loading">
            <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': loading }" />
            刷新
          </Button>
          <Button variant="default" size="sm" @click="handleSave" :disabled="saving || !isDirty">
            <Save class="w-4 h-4" :class="{ 'animate-spin': saving }" />
            {{ saving ? '保存中...' : '保存' }}
          </Button>
        </div>
      </div>
    </section>

    <!-- 加载状态 -->
    <div v-if="loading && !configSource" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2" style="border-color: var(--oc-accent);"></div>
        <p class="mt-3" style="color: var(--oc-text-muted);">加载配置中...</p>
      </div>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error && !configSource" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <div class="w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4" style="background: rgba(239, 68, 68, 0.1);">
          <AlertCircle class="w-8 h-8" style="color: var(--oc-error);" />
        </div>
        <h4 style="font-size: var(--text-base); color: var(--oc-text-primary);">加载失败</h4>
        <p class="mt-2 max-w-md mx-auto" style="color: var(--oc-text-muted); font-size: var(--text-sm);">
          {{ error }}
        </p>
        <Button variant="outline" size="sm" @click="handleRefresh" class="mt-4">
          <RefreshCw class="w-4 h-4" />
          重试
        </Button>
      </div>
    </div>

    <!-- 配置内容 -->
    <div v-else class="flex-1 overflow-auto space-y-4 pr-1">

      <!-- ============================================================ -->
      <!-- 基础网络配置 -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <Network class="w-4 h-4" style="color: var(--oc-accent);" />
          基础网络配置
        </h4>

        <div class="grid grid-cols-2 gap-4">
          <!-- 端口 -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">监听端口</span>
              <HelpTooltip title="监听端口" content="网关监听的端口号。默认 19633。修改后需要重启服务生效。" />
            </div>
            <Input
              v-model.number="gatewayPort"
              type="number"
              placeholder="19633"
              class="h-8 text-xs"
              :min="1024"
              :max="65535"
              @input="isDirty = true"
            />
          </div>

          <!-- 运行模式 -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">运行模式</span>
              <HelpTooltip title="运行模式" content="local = 本地开发模式，remote = 远程部署模式。" />
            </div>
            <StyledSelect
              v-model="gatewayMode"
              :options="modeOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- 绑定地址 -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">绑定地址</span>
              <HelpTooltip title="绑定地址" content="loopback = 仅本机 127.0.0.1 可访问（推荐）。0.0.0.0 = 所有网络接口均可访问，需确保有防火墙保护。" />
            </div>
            <StyledSelect
              v-model="gatewayBind"
              :options="bindOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- Tailscale -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">Tailscale</span>
              <HelpTooltip title="Tailscale" content="通过 Tailscale 组网，让远程设备安全访问本地网关服务。" />
            </div>
            <StyledSelect
              v-model="tailscaleMode"
              :options="tailscaleOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- 热重载模式 -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">热重载模式</span>
              <HelpTooltip title="热重载模式" content="配置文件变更时的应用策略。hybrid（推荐）安全改动即时生效，关键改动自动重启。需要手动重启的配置项：端口、绑定、认证、TLS、插件。" />
            </div>
            <StyledSelect
              v-model="hotReloadMode"
              :options="hotReloadModeOptions"
              @change="isDirty = true"
            />
          </div>
        </div>
      </section>

      <!-- ============================================================ -->
      <!-- 认证配置 -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <Shield class="w-4 h-4" style="color: var(--oc-accent);" />
          认证配置
        </h4>

        <div class="grid grid-cols-2 gap-4">
          <!-- 认证模式 -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">认证模式</span>
              <HelpTooltip title="认证模式" content="token = 使用 Token 验证请求（推荐）。none = 关闭认证（仅适用于本地开发环境）。" />
            </div>
            <StyledSelect
              v-model="authMode"
              :options="authModeOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- Token -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">认证 Token</span>
              <HelpTooltip title="认证 Token" content="用于 API 请求认证的 Token。留空则不设置。修改后需要重启服务生效。" />
            </div>
            <Input
              v-model="authToken"
              type="password"
              placeholder="留空则不修改"
              class="h-8 text-xs"
              :disabled="authMode === 'none'"
              @input="isDirty = true"
            />
          </div>
        </div>
      </section>

      <!-- ============================================================ -->
      <!-- 安全策略 -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <Ban class="w-4 h-4" style="color: var(--oc-accent);" />
          安全策略
        </h4>

        <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
          <div class="flex items-center gap-1 mb-2">
            <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">禁止的敏感命令</span>
            <HelpTooltip title="禁止的敏感命令" content="Agent 无法执行的敏感操作命令列表，如 camera.snap（摄像头拍照）等。添加后可防止 Agent 访问危险功能。" />
          </div>
          <div class="flex flex-wrap gap-1.5 min-h-[28px]">
            <Badge
              v-for="item in denyCommands" :key="item" variant="destructive"
              class="cursor-pointer group"
              @click="denyCommandsTag.pop(item)"
            >
              {{ item }}
              <Trash2 class="w-3 h-3 opacity-60 group-hover:opacity-100" />
            </Badge>
            <span v-if="denyCommands.length === 0" class="text-xs italic" style="color: var(--oc-text-muted);">
              未设置禁令
            </span>
          </div>
          <div class="flex gap-1 mt-1.5">
            <Input
              v-model="denyCommandsInput"
              placeholder="添加命令名..."
              class="h-7 text-xs flex-1"
              @keydown="denyCommandsTag.onKeydown($event)"
            />
            <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="denyCommandsTag.push()">+</Button>
          </div>
        </div>
      </section>

      <!-- ============================================================ -->
      <!-- Auth Profiles 认证配置管理 -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <KeyRound class="w-4 h-4" style="color: var(--oc-accent);" />
          Auth Profiles 认证配置
        </h4>

        <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
          <div class="flex items-center gap-1 mb-3">
            <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">认证 Profile 列表</span>
            <HelpTooltip title="Auth Profiles" content="管理多套认证凭据配置。每个 Profile 可以有独立的 provider 和 mode，用于多环境/多账号切换。Provider ID 需要对应 models.providers 中的配置。" />
          </div>

          <!-- Profile 列表 -->
          <div v-if="authProfiles.length > 0" class="space-y-2 mb-3">
            <div
              v-for="profile in authProfiles"
              :key="profile.id"
              class="flex items-center gap-3 px-3 py-2 rounded-lg"
              style="background: var(--oc-item-hover);"
            >
              <code class="text-xs font-mono min-w-[100px]" style="color: var(--oc-text-primary);">{{ profile.id }}</code>
              <Badge v-if="profile.provider" variant="outline" class="text-xs">{{ profile.provider }}</Badge>
              <Badge v-if="profile.mode" variant="accent" class="text-xs">{{ profile.mode }}</Badge>
              <div class="flex-1"></div>
              <button
                class="p-1 rounded hover:bg-red-500/10 transition-colors"
                @click="removeAuthProfile(profile.id)"
              >
                <Trash2 class="w-3.5 h-3.5" style="color: var(--oc-error);" />
              </button>
            </div>
          </div>
          <p v-else class="text-xs mb-3" style="color: var(--oc-text-muted);">尚未配置认证 Profile</p>

          <!-- 添加新 Profile -->
          <div class="flex items-end gap-2 pt-2" style="border-top: 1px solid var(--oc-divider);">
            <div class="flex-1">
              <span class="text-xs block mb-1" style="color: var(--oc-text-muted);">Profile ID</span>
              <Input
                v-model="newProfileId"
                placeholder="如 default, production"
                class="h-7 text-xs"
                @keydown.enter="addAuthProfile"
              />
            </div>
            <div class="flex-1">
              <span class="text-xs block mb-1" style="color: var(--oc-text-muted);">Provider</span>
              <Input
                v-model="newProfileProvider"
                placeholder="凭据提供商（可选）"
                class="h-7 text-xs"
                @keydown.enter="addAuthProfile"
              />
            </div>
            <div class="w-32">
              <span class="text-xs block mb-1" style="color: var(--oc-text-muted);">Mode</span>
              <StyledSelect
                v-model="newProfileMode"
                :options="authProfileModeOptions"
                placeholder="可选"
              />
            </div>
            <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="addAuthProfile">
              <Plus class="w-3.5 h-3.5" />
            </Button>
          </div>
        </div>
      </section>

      <!-- 底部：配置文件路径 -->
      <div v-if="configPath" class="px-2 py-2">
        <p class="text-xs" style="color: var(--oc-text-muted);">
          配置文件: {{ configPath }}
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-card {
  transition: box-shadow 0.15s ease;
}

.config-card:hover {
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.06);
}
</style>
