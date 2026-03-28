<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  ChevronDown,
  ExternalLink,
  Loader2,
  Monitor,
  Moon,
  RefreshCw,
  Settings2,
  Sun,
  Wifi,
} from 'lucide-vue-next'
import StatusDashboard from './components/pages/StatusDashboard.vue'
import InstallPage from './components/pages/InstallPage.vue'
import ConfigPage from './components/pages/ConfigPage.vue'
import QuickSetupGuide from './components/pages/QuickSetupGuide.vue'
import BindingsPage from './components/pages/BindingsPage.vue'
import MessageChannelsPage from './components/pages/MessageChannelsPage.vue'
import DiagnosticsPage from './components/pages/DiagnosticsPage.vue'
import SkillPresetsPage from './components/pages/SkillPresetsPage.vue'
import AgentWorkspacesPage from './components/pages/AgentWorkspacesPage.vue'
import CronJobsPage from './components/pages/CronJobsPage.vue'
import ChatArea from './components/ai-assistant/ChatArea.vue'
import SshConnectModal from './components/SshConnectModal.vue'
import SshFingerprintDialog from './components/SshFingerprintDialog.vue'
import Button from './components/ui/Button.vue'
import Card from './components/ui/Card.vue'
import Input from './components/ui/Input.vue'
import Toast from './components/ui/Toast.vue'

// 新的仪表盘组件
import DashboardGrid from './components/dashboard/DashboardGrid.vue'
import TopBar from './components/dashboard/TopBar.vue'
import DockBar from './components/dashboard/DockBar.vue'
import { deriveAppState } from './domain/appState'
import { NAV_ITEMS, type NavPage } from './domain/navigation'
import { isPrimaryModelPlaceholder } from './domain/configValidation'
import { shouldShowDashboardButton } from './domain/dashboardVisibility'
import {
  resolveGateTopbarTitle,
  shouldRenderQuickSetupCloseAction,
  shouldRenderQuickSetupGuide,
  shouldRenderSidebar,
  shouldUseFixedMainContentLayout,
} from './domain/gateInstallLayout'
import {
  clearQuickSetupSession,
  loadQuickSetupSession,
  shouldClearQuickSetupSessionAfterInstall,
  shouldClearQuickSetupSessionForEnvironment,
} from './domain/quickSetupSession'
import { DEFAULT_GATEWAY_READY_OPTIONS, waitForGatewayReady } from './domain/gatewayStartup'
import {
  OPENCLAW_UNINSTALL_CONFIRM_PHRASE,
  canConfirmOpenClawUninstallPhrase,
  resolveOpenClawUninstallActionState,
  resolveOpenClawUninstallCleanupItems,
  shouldShowOpenClawUninstallAction as shouldRenderOpenClawUninstallAction,
} from './domain/openclawUninstall'
import { resolveAsyncButtonLabel, resolveAsyncButtonState, runAsyncOnce } from './domain/asyncButtonState'
import { formatOpenClawVersionLabel } from './domain/openclawVersionLabel'
import { shouldShowOpenConfigFileAction } from './domain/sidebarConfigStatus'
import appIcon from './assets/app-icon.png'
import type {
  ConfigFileInfo,
  EnvironmentStatus,
  FingerprintInfo,
  OpenClawConfig,
  SshProfile,
} from './types/config'

type EnvMode = 'local' | 'ssh'
type ThemeMode = 'system' | 'light' | 'dark'
type OpenClawUninstallStep = 'confirm' | 'phrase' | 'config'

interface EnvironmentInfo {
  mode: EnvMode
  label: string
  sshProfile?: SshProfile
}

const environments = ref<EnvironmentInfo[]>([{ mode: 'local', label: '本地环境' }])
const currentEnvIndex = ref(0)
const currentEnv = computed(() => environments.value[currentEnvIndex.value])
const targetMode = ref<EnvMode | null>(null)

const showEnvDropdown = ref(false)
const showSshModal = ref(false)
const showFingerprintDialog = ref(false)
const sshConnected = ref(false)
const sshFingerprint = ref<FingerprintInfo | null>(null)
const sshFingerprintCallback = ref<(() => void) | null>(null)
const themeStorageKey = 'clawlite.theme.mode'
const browserDefaultProfile = 'openclaw'
const isWindows = navigator.userAgent.toLowerCase().includes('windows')
const themeMode = ref<ThemeMode>('system')
const themeModeCycle: ThemeMode[] = ['system', 'light', 'dark']
const browserDefaultProfileEnabled = ref(false)
const browserSettingLoading = ref(false)
const browserSettingSaving = ref(false)
const browserSettingError = ref('')
const browserSettingPath = ref('')
const browserSettingReady = ref(false)
const configFilePath = ref('')
const uninstallOpenClawStep = ref<OpenClawUninstallStep | null>(null)
const uninstallOpenClawInput = ref('')
const uninstallOpenClawLoading = ref(false)

const envStatus = ref<EnvironmentStatus | null>(null)
const configLoaded = ref(false)
const primaryModelValid = ref(false)
const gatewayReachable = ref(false)
const gatewayChecking = ref(false)
const gatewayServiceInstalled = ref(true)
const lastActionFailed = ref(false)
const pendingToolId = ref<string | null>(null)
const dashboardOpening = ref(false)
const environmentRefreshing = ref(false)

// 绑定和渠道统计
const bindingsCount = ref(0)
const activeBindingsCount = ref(0)
const channelsEnabled = ref(0)
const totalChannels = ref(0)

// Skill 预设、Agent 工作空间、Cron 定时任务统计
const presetsCount = ref(0)
const activePresetsCount = ref(0)
const workspacesCount = ref(0)
const activeWorkspacesCount = ref(0)
const totalJobs = ref(0)
const enabledJobs = ref(0)
const jobsWithErrors = ref(0)

const loading = ref(false)
const loadingMessage = ref('加载中...')
const activeNav = ref<NavPage>('overview')
const quickSetupDebugOpen = ref(false)
const quickSetupResumePending = ref(Boolean(loadQuickSetupSession()))

const toast = ref<{ type: 'success' | 'error'; message: string } | null>(null)
let toastTimer: ReturnType<typeof setTimeout> | null = null

const openclawInstalled = computed(() => envStatus.value?.openclaw.installed ?? false)
const envConnected = computed(() => (currentEnv.value.mode === 'local' ? true : sshConnected.value))

const appState = computed(() =>
  deriveAppState({
    envConnected: envConnected.value,
    openclawInstalled: openclawInstalled.value,
    configLoaded: configLoaded.value,
    primaryModelValid: primaryModelValid.value,
    gatewayReachable: gatewayReachable.value,
    lastActionFailed: lastActionFailed.value,
  })
)

type GateState = 'NO_TARGET' | 'NEED_INSTALL' | 'NEED_CONFIG' | null

const gateState = computed<GateState>(() => {
  if (appState.value === 'NO_TARGET') return 'NO_TARGET'
  if (appState.value === 'NEED_INSTALL') return 'NEED_INSTALL'
  if (appState.value === 'NEED_CONFIG') return 'NEED_CONFIG'
  return null
})

const isGateActive = computed(() => gateState.value !== null)
const showDashboardButton = computed(() => shouldShowDashboardButton(gateState.value))
const dashboardButtonState = computed(() =>
  resolveAsyncButtonState({
    loading: dashboardOpening.value,
  })
)
const dashboardButtonLabel = computed(() =>
  resolveAsyncButtonLabel({
    loading: dashboardOpening.value,
    label: 'Dashboard',
    loadingLabel: '打开中...',
  })
)
const refreshEnvironmentButtonState = computed(() =>
  resolveAsyncButtonState({
    loading: environmentRefreshing.value,
    baseDisabled: loading.value && !environmentRefreshing.value,
  })
)

const stateTextMap: Record<string, string> = {
  NO_TARGET: '未连接环境',
  NEED_INSTALL: '待安装',
  NEED_CONFIG: '待配置',
  READY: '可用',
  DEGRADED: '可用但降级',
  ERROR: '错误',
}

const navMeta: Record<NavPage, { title: string; subtitle: string }> = {
  overview: {
    title: '工作台',
    subtitle: '快速确认可用状态与下一步操作。',
  },
  'ai-config': {
    title: '模型配置',
    subtitle: '保存配置文件并调整模型路由。',
  },
  bindings: {
    title: '绑定管理',
    subtitle: '配置 Agent 与消息渠道的绑定关系。',
  },
  diagnostics: {
    title: '服务诊断',
    subtitle: '遇到异常时快速定位问题并执行修复动作。',
  },
  channels: {
    title: '消息渠道',
    subtitle: '通知增强能力，不阻塞 OpenClaw 主流程。',
  },
  'skill-presets': {
    title: '技能预设',
    subtitle: '发现和安装 AI Agent 技能扩展。',
  },
  'agent-workspaces': {
    title: 'Agent Workspaces',
    subtitle: '管理和使用职场Agent岗位预设。',
  },
  'cron-jobs': {
    title: 'Cron 定时任务',
    subtitle: '管理 OpenClaw 的 Cron 定时任务配置。',
  },
  settings: {
    title: '系统设置',
    subtitle: '管理环境、SSH Profile 与连接偏好。',
  },
}

const pageMeta = computed(() => navMeta[activeNav.value])
const quickSetupForcedOpen = computed(() => quickSetupDebugOpen.value || quickSetupResumePending.value)
const shouldShowSidebar = computed(() => shouldRenderSidebar(isGateActive.value, quickSetupForcedOpen.value))
const shouldShowQuickSetupCloseAction = computed(() =>
  shouldRenderQuickSetupCloseAction(isGateActive.value, quickSetupDebugOpen.value)
)
const topbarTitle = computed(() => {
  if (quickSetupForcedOpen.value) return '安装与接入 · 快速引导调试'
  if (!isGateActive.value) return pageMeta.value.title
  return resolveGateTopbarTitle(gateState.value, targetMode.value)
})
const fixedMainContentLayout = computed(() =>
  shouldUseFixedMainContentLayout(
    isGateActive.value,
    gateState.value,
    targetMode.value,
    activeNav.value,
    quickSetupForcedOpen.value,
  )
)
const shouldShowQuickSetupGuide = computed(() =>
  shouldRenderQuickSetupGuide(
    isGateActive.value,
    gateState.value,
    targetMode.value,
    quickSetupForcedOpen.value,
    Boolean(envStatus.value),
  )
)
const themeModeLabel = computed(() => {
  if (themeMode.value === 'light') return '浅色'
  if (themeMode.value === 'dark') return '深色'
  return '跟随系统'
})
const themeModeIcon = computed(() => {
  if (themeMode.value === 'light') return Sun
  if (themeMode.value === 'dark') return Moon
  return Monitor
})
const themeButtonTitle = computed(() => `主题：${themeModeLabel.value}（点击切换）`)
const globalVersionText = computed(() => formatOpenClawVersionLabel(envStatus.value?.openclaw.version))
const configStatusText = computed(() => {
  if (!openclawInstalled.value) return '未安装'
  if (!configLoaded.value) return '未加载'
  if (!primaryModelValid.value) return '主模型无效'
  return '配置有效'
})
const showOpenConfigFileAction = computed(() =>
  shouldShowOpenConfigFileAction({
    envMode: currentEnv.value.mode,
    configStatusText: configStatusText.value,
    configFilePath: configFilePath.value,
  })
)
const browserSettingStatusText = computed(() => {
  if (browserSettingLoading.value) return '加载中'
  if (browserSettingSaving.value) return '保存中'
  return browserDefaultProfileEnabled.value ? '已开启' : '已关闭'
})
const browserSettingSwitchDisabled = computed(() => {
  if (browserSettingLoading.value || browserSettingSaving.value) return true
  if (!openclawInstalled.value) return true
  if (currentEnv.value.mode === 'ssh' && !sshConnected.value) return true
  return !browserSettingReady.value
})
const showOpenClawUninstallAction = computed(() =>
  shouldRenderOpenClawUninstallAction({
    envMode: currentEnv.value.mode,
  })
)
const openClawUninstallActionState = computed(() =>
  resolveOpenClawUninstallActionState({
    envMode: currentEnv.value.mode,
    openclawInstalled: openclawInstalled.value,
    loading: uninstallOpenClawLoading.value,
  })
)
const uninstallOpenClawPhraseValid = computed(() =>
  canConfirmOpenClawUninstallPhrase(uninstallOpenClawInput.value)
)
const currentSystemOs = computed<'windows' | 'macos' | 'linux'>(() => {
  if (envStatus.value?.system.os) return envStatus.value.system.os
  if (isWindows) return 'windows'
  return navigator.userAgent.toLowerCase().includes('mac') ? 'macos' : 'linux'
})
const uninstallCleanupItemsWithoutConfig = computed(() =>
  resolveOpenClawUninstallCleanupItems({ os: currentSystemOs.value, removeConfigDir: false })
)
const uninstallCleanupItemsWithConfig = computed(() =>
  resolveOpenClawUninstallCleanupItems({ os: currentSystemOs.value, removeConfigDir: true })
)
const isThemeMode = (raw: string | null): raw is ThemeMode =>
  raw === 'system' || raw === 'light' || raw === 'dark'

const applyThemeMode = (mode: ThemeMode) => {
  const root = document.documentElement
  if (mode === 'system') {
    root.removeAttribute('data-theme')
    return
  }
  root.setAttribute('data-theme', mode)
}

const loadThemeMode = () => {
  try {
    const raw = localStorage.getItem(themeStorageKey)
    if (isThemeMode(raw)) {
      themeMode.value = raw
    }
  } catch {
    // ignored
  }
  applyThemeMode(themeMode.value)
}

const cycleThemeMode = () => {
  const currentIndex = themeModeCycle.indexOf(themeMode.value)
  const nextIndex = (currentIndex + 1) % themeModeCycle.length
  themeMode.value = themeModeCycle[nextIndex]
}

watch(themeMode, (mode) => {
  applyThemeMode(mode)
  try {
    localStorage.setItem(themeStorageKey, mode)
  } catch {
    // ignored
  }
})

const showToast = (type: 'success' | 'error', message: string) => {
  if (toastTimer) clearTimeout(toastTimer)
  toast.value = { type, message }
  toastTimer = setTimeout(() => {
    toast.value = null
  }, 3200)
}

const closeToast = () => {
  if (toastTimer) clearTimeout(toastTimer)
  toast.value = null
}

const resetOpenClawUninstallFlow = () => {
  uninstallOpenClawStep.value = null
  uninstallOpenClawInput.value = ''
}

const closeOpenClawUninstallFlow = () => {
  if (uninstallOpenClawLoading.value) return
  resetOpenClawUninstallFlow()
}

const openOpenClawUninstallFlow = () => {
  if (openClawUninstallActionState.value.disabled) return
  uninstallOpenClawInput.value = ''
  uninstallOpenClawStep.value = 'confirm'
}

const continueOpenClawUninstallFlow = () => {
  if (uninstallOpenClawLoading.value) return
  uninstallOpenClawInput.value = ''
  uninstallOpenClawStep.value = 'phrase'
}

const confirmOpenClawUninstallPhraseStep = () => {
  if (uninstallOpenClawLoading.value || !uninstallOpenClawPhraseValid.value) return
  uninstallOpenClawStep.value = 'config'
}

const copyTextToClipboard = async (value: string) => {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(value)
    return
  }

  const textarea = document.createElement('textarea')
  textarea.value = value
  textarea.setAttribute('readonly', 'true')
  textarea.style.position = 'fixed'
  textarea.style.opacity = '0'
  document.body.appendChild(textarea)
  textarea.select()
  const success = document.execCommand('copy')
  document.body.removeChild(textarea)
  if (!success) {
    throw new Error('copy_failed')
  }
}

const copyOpenClawUninstallPhrase = async () => {
  try {
    await copyTextToClipboard(OPENCLAW_UNINSTALL_CONFIRM_PHRASE)
    showToast('success', '确认短语已复制')
  } catch {
    showToast('error', '复制失败，请手动复制')
  }
}
const runOpenClawUninstall = async (removeConfigDir: boolean) => {
  if (openClawUninstallActionState.value.disabled) return

  uninstallOpenClawLoading.value = true
  loading.value = true
  loadingMessage.value = removeConfigDir
    ? '正在卸载 OpenClaw 并删除 ~/.openclaw...'
    : '正在卸载 OpenClaw...'

  try {
    const result = await invoke<string>('uninstall_openclaw', { removeConfigDir })
    resetOpenClawUninstallFlow()
    await checkEnvironment()
    showToast('success', result)
  } catch (error) {
    showToast('error', `卸载 OpenClaw 失败: ${String(error)}`)
  } finally {
    uninstallOpenClawLoading.value = false
    loading.value = false
    loadingMessage.value = '加载中...'
  }
}

const runGatewayStartCommand = async () => {
  if (currentEnv.value.mode === 'ssh') {
    if (!sshConnected.value) {
      throw new Error('SSH 未连接，无法启动远程网关')
    }
    await invoke('ssh_start_gateway')
    return
  }
  await invoke('start_gateway')
}

const checkGatewayHealth = async (): Promise<boolean> => {
  try {
    if (currentEnv.value.mode === 'ssh') {
      if (!sshConnected.value) return false
      return await invoke<boolean>('ssh_health_check')
    }
    return await invoke<boolean>('health_check_gateway')
  } catch {
    return false
  }
}

const waitForGatewayReadyWithMessage = async (
  message: string,
  maxAttempts = DEFAULT_GATEWAY_READY_OPTIONS.maxAttempts,
  intervalMs = DEFAULT_GATEWAY_READY_OPTIONS.intervalMs
): Promise<boolean> => {
  let attempts = 0
  return waitForGatewayReady(
    async () => {
      attempts += 1
      loadingMessage.value = `${message}（${attempts}/${maxAttempts}）...`
      return checkGatewayHealth()
    },
    { maxAttempts, intervalMs }
  )
}

const markActionResult = (
  _action: string,
  ok: boolean,
  successMessage: string,
  errorMessage?: string
) => {
  lastActionFailed.value = !ok
  const detail = ok ? successMessage : errorMessage || '操作失败'
  showToast(ok ? 'success' : 'error', detail)
}

const runWithPendingTool = async (toolId: string, action: () => Promise<void>) => {
  if (pendingToolId.value) {
    return
  }

  pendingToolId.value = toolId
  try {
    await action()
  } finally {
    pendingToolId.value = null
  }
}

const syncConfigSignals = async () => {
  console.log('[syncConfigSignals] 开始执行', { openclawInstalled: openclawInstalled.value, currentMode: currentEnv.value.mode })

  if (!openclawInstalled.value) {
    console.log('[syncConfigSignals] OpenClaw 未安装，跳过所有检查')
    configLoaded.value = false
    primaryModelValid.value = false
    gatewayReachable.value = false
    configFilePath.value = ''
    // 重置统计数据
    bindingsCount.value = 0
    activeBindingsCount.value = 0
    channelsEnabled.value = 0
    totalChannels.value = 0
    presetsCount.value = 0
    activePresetsCount.value = 0
    workspacesCount.value = 0
    activeWorkspacesCount.value = 0
    totalJobs.value = 0
    enabledJobs.value = 0
    jobsWithErrors.value = 0
    return
  }

  console.log('[syncConfigSignals] OpenClaw 已安装，继续执行检查')

  let config: OpenClawConfig | null = null

  try {
    if (currentEnv.value.mode === 'ssh' && sshConnected.value) {
      const results = await invoke<Array<{ path: string }>>('ssh_search_config')
      if (!results.length) {
        configLoaded.value = false
        primaryModelValid.value = false
      } else {
        const raw = await invoke<string>('ssh_read_file', { path: results[0].path })
        config = JSON.parse(raw) as OpenClawConfig
        const primary = config.agents?.defaults?.model?.primary
        configLoaded.value = true
        primaryModelValid.value = !isPrimaryModelPlaceholder(primary)
      }
      configFilePath.value = ''
    } else {
      const [configData, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')
      config = configData
      const primary = config.agents?.defaults?.model?.primary
      configLoaded.value = true
      primaryModelValid.value = !isPrimaryModelPlaceholder(primary)
      configFilePath.value = info.path
    }
  } catch {
    console.error('[syncConfigSignals] 配置加载失败')
    configLoaded.value = false
    primaryModelValid.value = false
    configFilePath.value = ''
    config = null
  }

  // 解析绑定和渠道统计数据
  if (config) {
    try {
      // 解析绑定数据
      const bindings = await invoke<any[]>('parse_bindings', { config })
      bindingsCount.value = bindings.length
      // 统计活跃绑定（有 peerId 的视为活跃）
      activeBindingsCount.value = bindings.filter((b: any) => b.peerId && b.peerId.length > 0).length
    } catch {
      bindingsCount.value = 0
      activeBindingsCount.value = 0
    }

    try {
      // 解析渠道数据
      const channels = config.channels as Record<string, unknown> | undefined
      if (channels && typeof channels === 'object') {
        const channelKeys = Object.keys(channels).filter(key => {
          // 过滤掉 connector 类型的特殊键
          return !key.includes('-connector') && key !== 'dingtalk-connector'
        })
        totalChannels.value = channelKeys.length

        // 统计已启用的渠道（有配置信息的渠道）
        let enabledCount = 0
        for (const key of channelKeys) {
          const channel = channels[key]
          if (channel && typeof channel === 'object' && !Array.isArray(channel)) {
            const channelObj = channel as Record<string, unknown>
            // 检查是否有有效的配置（不同渠道有不同的必填字段）
            const hasValidConfig =
              ('appId' in channelObj && 'appSecret' in channelObj) || // 飞书、企业微信等
              ('token' in channelObj) || // Telegram、Discord 等
              ('botToken' in channelObj) || // QQ机器人
              ('enabled' in channelObj && channelObj.enabled === true) // 通用启用标记

            if (hasValidConfig) {
              enabledCount++
            }
          }
        }
        channelsEnabled.value = enabledCount
      } else {
        totalChannels.value = 0
        channelsEnabled.value = 0
      }
    } catch {
      channelsEnabled.value = 0
      totalChannels.value = 0
    }
  } else {
    bindingsCount.value = 0
    activeBindingsCount.value = 0
    channelsEnabled.value = 0
    totalChannels.value = 0
  }

  // 解析 Skill 预设、Agent 工作空间、Cron 定时任务统计数据
  try {
    // 获取 Cron 定时任务统计
    if (currentEnv.value.mode === 'ssh' && sshConnected.value) {
      const jobs = await invoke<any[]>('ssh_get_cron_jobs')
      totalJobs.value = jobs.length
      enabledJobs.value = jobs.filter((j: any) => j.enabled).length
      jobsWithErrors.value = jobs.filter((j: any) =>
        j.state?.lastRunStatus === 'error' || (j.state?.consecutiveErrors || 0) > 0
      ).length
    } else {
      const jobs = await invoke<any[]>('get_cron_jobs')
      totalJobs.value = jobs.length
      enabledJobs.value = jobs.filter((j: any) => j.enabled).length
      jobsWithErrors.value = jobs.filter((j: any) =>
        j.state?.lastRunStatus === 'error' || (j.state?.consecutiveErrors || 0) > 0
      ).length
    }
  } catch (error) {
    console.error('[Cron Jobs] 加载失败:', error)
    totalJobs.value = 0
    enabledJobs.value = 0
    jobsWithErrors.value = 0
  }

  try {
    // 获取 Skill 预设统计
    if (currentEnv.value.mode === 'ssh' && sshConnected.value) {
      // SSH 模式暂不支持
      presetsCount.value = 0
      activePresetsCount.value = 0
    } else {
      const skills = await invoke<any[]>('get_skills_with_status')
      presetsCount.value = skills.length
      // 统计已安装并启用的预设
      activePresetsCount.value = skills.filter((s: any) => s.installed && s.enabled).length
    }
  } catch (error) {
    console.error('[Skill Presets] 加载失败:', error)
    presetsCount.value = 0
    activePresetsCount.value = 0
  }

  try {
    // 获取 Agent 工作空间统计
    if (currentEnv.value.mode === 'ssh' && sshConnected.value) {
      // SSH 模式暂不支持
      workspacesCount.value = 0
      activeWorkspacesCount.value = 0
    } else {
      const workspaces = await invoke<any[]>('get_all_agent_workspaces')
      workspacesCount.value = workspaces.length
      // 统计活跃的工作空间（默认都视为可用，因为没有启用/禁用状态）
      activeWorkspacesCount.value = workspaces.length
    }
  } catch (error) {
    console.error('[Agent Workspaces] 加载失败:', error)
    workspacesCount.value = 0
    activeWorkspacesCount.value = 0
  }

  try {
    console.log('[syncConfigSignals] 开始网关健康检查', {
      mode: currentEnv.value.mode,
      sshConnected: sshConnected.value,
      before: gatewayReachable.value
    })

    if (currentEnv.value.mode === 'ssh' && sshConnected.value) {
      gatewayReachable.value = await invoke<boolean>('ssh_health_check')
      console.log('[syncConfigSignals] SSH 健康检查结果:', gatewayReachable.value)
    } else {
      gatewayReachable.value = await invoke<boolean>('health_check_gateway')
      console.log('[syncConfigSignals] 本地健康检查结果:', gatewayReachable.value)
    }
  } catch (error) {
    console.error('[syncConfigSignals] 健康检查异常:', error)
    gatewayReachable.value = false
  }

  console.log('[syncConfigSignals] 执行完成', { gatewayReachable: gatewayReachable.value })
}

const openConfigFile = async () => {
  if (!showOpenConfigFileAction.value) return

  try {
    await invoke('open_path_in_default_app', { path: configFilePath.value })
  } catch (error) {
    showToast('error', `打开配置文件失败: ${String(error)}`)
  }
}

const syncGatewayServiceInstallState = async () => {
  if (!isWindows || currentEnv.value.mode !== 'local' || !openclawInstalled.value) {
    gatewayServiceInstalled.value = true
    return
  }

  try {
    gatewayServiceInstalled.value = await invoke<boolean>('is_gateway_service_installed')
  } catch {
    gatewayServiceInstalled.value = false
  }
}

const checkEnvironment = async () => {
  console.log('[checkEnvironment] 开始执行', { initialCheck: envStatus.value === null, currentMode: currentEnv.value.mode })

  // 只在非初始加载时显示 loading
  const initialCheck = envStatus.value === null
  if (!initialCheck) {
    loading.value = true
  }

  // 开始检测网关状态
  gatewayChecking.value = true

  try {
    console.log('[checkEnvironment] 调用环境检测命令', { mode: currentEnv.value.mode, sshConnected: sshConnected.value })

    if (currentEnv.value.mode === 'ssh') {
      if (!sshConnected.value) {
        console.log('[checkEnvironment] SSH 未连接，跳过检测')
        envStatus.value = null
        configLoaded.value = false
        primaryModelValid.value = false
        gatewayReachable.value = false
        gatewayServiceInstalled.value = true
      } else {
        envStatus.value = await invoke<EnvironmentStatus>('ssh_check_environment')
        console.log('[checkEnvironment] SSH 环境检测结果:', {
          openclawInstalled: envStatus.value?.openclaw?.installed,
          nodeInstalled: envStatus.value?.node?.installed
        })
      }
    } else {
      envStatus.value = await invoke<EnvironmentStatus>('check_environment')
      console.log('[checkEnvironment] 本地环境检测结果:', {
        openclawInstalled: envStatus.value?.openclaw?.installed,
        nodeInstalled: envStatus.value?.node?.installed
      })
    }

    if (envStatus.value && shouldClearQuickSetupSessionForEnvironment(currentEnv.value.mode, envStatus.value.openclaw.installed)) {
      clearQuickSetupSession()
      quickSetupResumePending.value = false
      quickSetupDebugOpen.value = false
    }

    console.log('[checkEnvironment] 准备调用 syncConfigSignals')
    await syncConfigSignals()
    await syncGatewayServiceInstallState()
  } catch (error) {
    console.error('[checkEnvironment] 环境检测失败:', error)
    envStatus.value = null
    configLoaded.value = false
    primaryModelValid.value = false
    gatewayReachable.value = false
    gatewayServiceInstalled.value = true
    markActionResult('环境检测', false, '', '环境检测失败')
  } finally {
    loading.value = false
    gatewayChecking.value = false
    console.log('[checkEnvironment] 执行完成', { gatewayReachable: gatewayReachable.value })
  }
}

const refreshEnvironment = async () =>
  runAsyncOnce({
    isRunning: () => environmentRefreshing.value,
    setRunning: (running) => {
      environmentRefreshing.value = running
    },
    action: checkEnvironment,
  })

const pageBlockedReason = (target: NavPage) => {
  // 这些页面不需要门禁检查
  if (target === 'channels') return null
  if (target === 'skill-presets') return null

  if (isGateActive.value) {
    if (target === 'overview') {
      return null
    }
    return '当前处于门禁状态，请先完成前置步骤'
  }

  if (appState.value === 'NO_TARGET' && !['settings'].includes(target)) {
    return '请先连接环境'
  }

  if (appState.value === 'NEED_INSTALL' && !['settings'].includes(target)) {
    return '请先完成安装'
  }

  if (appState.value === 'NEED_CONFIG' && target === 'overview') {
    return '请先完成模型配置并验证'
  }

  return null
}

const navigateTo = (target: NavPage) => {
  const reason = pageBlockedReason(target)
  if (reason) {
    showToast('error', reason)
    return
  }

  if (target !== 'overview') {
    quickSetupDebugOpen.value = false
  }
  activeNav.value = target
}

// 卡片按钮操作处理
const handleCardAction = (cardId: NavPage, action: string) => {
  switch (cardId) {
    case 'overview':
      handleStatusCardAction(action)
      break
    case 'ai-config':
      handleConfigCardAction(action)
      break
    case 'bindings':
      handleBindingCardAction(action)
      break
    case 'diagnostics':
      handleDiagnosticsCardAction(action)
      break
    case 'channels':
      handleChannelsCardAction(action)
      break
    case 'settings':
      handleSettingsCardAction(action)
      break
  }
}

// 服务状态卡片操作处理
const handleStatusCardAction = (action: string) => {
  console.log('[handleStatusCardAction] 按钮点击:', { action, gatewayReachable: gatewayReachable.value })
  switch (action) {
    case 'details':
      navigateTo('overview')
      break
    case 'restart':
      openToolPanel('restart')
      break
    case 'start':
      openToolPanel('start')
      break
    case 'stop':
      openToolPanel('stop')
      break
  }
}

// 配置管理卡片操作处理
const handleConfigCardAction = (action: string) => {
  // 编辑配置和详情都跳转到 ai-config 页面
  navigateTo('ai-config')
}

// 绑定管理卡片操作处理
const handleBindingCardAction = (action: string) => {
  navigateTo('bindings')
  // TODO: 如果是 'add'，可能需要触发新建绑定的状态
}

// 诊断工具卡片操作处理
const handleDiagnosticsCardAction = (action: string) => {
  if (action === 'run') {
    openTool('doctor')
  } else {
    navigateTo('diagnostics')
  }
}

// 消息渠道卡片操作处理
const handleChannelsCardAction = (action: string) => {
  navigateTo('channels')
  // TODO: 如果是 'add'，可能需要触发添加渠道的状态
}

// 系统设置卡片操作处理
const handleSettingsCardAction = (action: string) => {
  // 所有设置操作都跳转到 settings 页面
  navigateTo('settings')
  // TODO: 根据 action 类型可能需要滚动到特定区域
}

const openQuickSetupDebug = async () => {
  targetMode.value = 'local'
  quickSetupDebugOpen.value = true
  if (!envStatus.value) {
    await checkEnvironment()
  }
}

watch(
  appState,
  (state) => {
    if (!targetMode.value && state !== 'NO_TARGET') {
      targetMode.value = currentEnv.value.mode
    }

    if (gateState.value === 'NO_TARGET' || gateState.value === 'NEED_INSTALL') {
      quickSetupDebugOpen.value = false
      activeNav.value = 'overview'
      return
    }

    if (gateState.value === 'NEED_CONFIG') {
      quickSetupDebugOpen.value = false
      activeNav.value = 'overview'
      return
    }

    // 不做错误态强制跳转，默认保持在工作台。
  },
  { immediate: true }
)

watch(
  [activeNav, currentEnvIndex, sshConnected, openclawInstalled],
  async ([nav]) => {
    if (nav !== 'settings') return
    await loadBrowserToolSetting()
  }
)

const handleGlobalClick = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  if (!target.closest('.env-dropdown-container')) {
    showEnvDropdown.value = false
  }
}

const loadSshProfiles = async () => {
  try {
    const profiles = await invoke<SshProfile[]>('ssh_load_profiles')
    const next = [{ mode: 'local', label: '本地环境' } as EnvironmentInfo]
    for (const profile of profiles) {
      next.push({
        mode: 'ssh',
        label: profile.name || `${profile.host}:${profile.port}`,
        sshProfile: profile,
      })
    }
    environments.value = next
  } catch {
    environments.value = [{ mode: 'local', label: '本地环境' }]
  }
}

const selectEnvironment = async (index: number) => {
  showEnvDropdown.value = false
  currentEnvIndex.value = index
  envStatus.value = null
  lastActionFailed.value = false

  const selected = environments.value[index]
  targetMode.value = selected.mode
  if (selected.mode === 'local') {
    if (sshConnected.value) {
      try {
        await invoke('ssh_disconnect')
      } catch {
        // ignored
      }
      sshConnected.value = false
    }
    await checkEnvironment()
    return
  }

  if (!sshConnected.value) {
    showSshModal.value = true
    return
  }

  await checkEnvironment()
}

const chooseLocalTarget = async () => {
  targetMode.value = 'local'
  await selectEnvironment(0)
}

const chooseSshTarget = async () => {
  targetMode.value = 'ssh'
  const sshIndex = environments.value.findIndex((item) => item.mode === 'ssh')
  if (sshIndex >= 0) {
    await selectEnvironment(sshIndex)
    return
  }
  showSshModal.value = true
}

const runManualConfig = async () => {
  try {
    await invoke('open_terminal_with_command', {
      command: 'openclaw onboard --install-daemon',
    })
    markActionResult('手动配置', true, '已打开终端执行 openclaw onboard --install-daemon')
  } catch (error) {
    markActionResult('手动配置', false, '', `打开终端失败: ${error}`)
  }
}

const applyDefaultConfig = async () => {
  loading.value = true
  loadingMessage.value = '正在写入默认配置...'
  try {
    await invoke('generate_default_config')
    loadingMessage.value = '正在安装网关服务...'
    await invoke('install_gateway_service')

    loadingMessage.value = '正在启动网关服务...'
    await runGatewayStartCommand()

    loadingMessage.value = '正在监控网关健康状态...'
    const ready = await waitForGatewayReadyWithMessage('正在等待网关启动')
    if (!ready) {
      throw new Error('网关在预期时间内未启动成功，请检查日志后重试')
    }

    loadingMessage.value = '正在进入工作台...'
    await checkEnvironment()
    activeNav.value = 'overview'
    markActionResult('默认配置', true, '默认配置完成，网关已启动，已进入工作台')
  } catch (error) {
    markActionResult('默认配置', false, '', `默认配置执行失败: ${error}`)
  } finally {
    loadingMessage.value = '加载中...'
    loading.value = false
  }
}

const addSshEnvironment = () => {
  showEnvDropdown.value = false
  showSshModal.value = true
}

interface BrowserSettingConfigSource {
  mode: 'local' | 'ssh'
  path: string
  config: OpenClawConfig
}

const resolveCurrentConfigSource = async (): Promise<BrowserSettingConfigSource> => {
  if (currentEnv.value.mode === 'ssh') {
    if (!sshConnected.value) {
      throw new Error('SSH 未连接')
    }
    const results = await invoke<Array<{ path: string }>>('ssh_search_config')
    if (!results.length) {
      throw new Error('未找到远程配置文件')
    }
    const path = results[0].path
    const raw = await invoke<string>('ssh_read_file', { path })
    const config = JSON.parse(raw) as OpenClawConfig
    return { mode: 'ssh', path, config }
  }

  const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')
  return {
    mode: 'local',
    path: info.path,
    config,
  }
}

const persistCurrentConfigSource = async (source: BrowserSettingConfigSource) => {
  if (source.mode === 'ssh') {
    await invoke('ssh_write_file', {
      path: source.path,
      content: JSON.stringify(source.config, null, 2),
    })
    return
  }
  await invoke('save_config', {
    config: source.config,
    path: source.path,
  })
}

const readBrowserDefaultProfileEnabled = (config: OpenClawConfig) => {
  const browserRaw = config.browser
  if (!browserRaw || typeof browserRaw !== 'object' || Array.isArray(browserRaw)) {
    return false
  }
  const defaultProfile = (browserRaw as Record<string, unknown>).defaultProfile
  return defaultProfile === browserDefaultProfile
}

const loadBrowserToolSetting = async () => {
  browserSettingError.value = ''
  browserSettingPath.value = ''
  browserSettingReady.value = false

  if (!openclawInstalled.value) {
    browserDefaultProfileEnabled.value = false
    browserSettingError.value = '当前环境未安装 OpenClaw，无法读取配置'
    return
  }

  if (currentEnv.value.mode === 'ssh' && !sshConnected.value) {
    browserDefaultProfileEnabled.value = false
    browserSettingError.value = 'SSH 未连接，无法读取远程配置'
    return
  }

  browserSettingLoading.value = true
  try {
    const source = await resolveCurrentConfigSource()
    browserSettingPath.value = source.path
    browserDefaultProfileEnabled.value = readBrowserDefaultProfileEnabled(source.config)
    browserSettingReady.value = true
  } catch (error) {
    browserDefaultProfileEnabled.value = false
    browserSettingError.value = String(error)
  } finally {
    browserSettingLoading.value = false
  }
}

const toggleBrowserDefaultProfile = async () => {
  if (browserSettingSwitchDisabled.value) return

  const next = !browserDefaultProfileEnabled.value
  const previous = browserDefaultProfileEnabled.value
  browserSettingSaving.value = true
  browserSettingError.value = ''
  browserDefaultProfileEnabled.value = next

  try {
    const source = await resolveCurrentConfigSource()
    const browserRaw = source.config.browser
    const browserConfig: Record<string, unknown> =
      browserRaw && typeof browserRaw === 'object' && !Array.isArray(browserRaw)
        ? { ...(browserRaw as Record<string, unknown>) }
        : {}

    if (next) {
      browserConfig.defaultProfile = browserDefaultProfile
    } else {
      delete browserConfig.defaultProfile
    }

    source.config.browser = browserConfig
    await persistCurrentConfigSource(source)
    browserSettingPath.value = source.path
    showToast('success', next ? '已开启浏览器默认 Profile（openclaw）' : '已关闭浏览器默认 Profile')
  } catch (error) {
    browserDefaultProfileEnabled.value = previous
    browserSettingError.value = String(error)
    showToast('error', `保存浏览器工具设置失败: ${error}`)
  } finally {
    browserSettingSaving.value = false
  }
}

const handleFingerprint = (info: FingerprintInfo, onConfirm: () => void) => {
  if (info.isKnown) {
    onConfirm()
    return
  }
  sshFingerprint.value = info
  sshFingerprintCallback.value = onConfirm
  showFingerprintDialog.value = true
}

const confirmFingerprint = async () => {
  showFingerprintDialog.value = false
  if (sshFingerprint.value) {
    try {
      await invoke('ssh_save_fingerprint', { fingerprint: sshFingerprint.value.sha256 })
    } catch {
      // ignored
    }
  }
  sshFingerprintCallback.value?.()
  sshFingerprint.value = null
  sshFingerprintCallback.value = null
}

const rejectFingerprint = async () => {
  showFingerprintDialog.value = false
  sshFingerprint.value = null
  sshFingerprintCallback.value = null
  try {
    await invoke('ssh_disconnect')
  } catch {
    // ignored
  }
  sshConnected.value = false
  showToast('error', '已拒绝 SSH 指纹，连接已取消')
}

const handleSshConnected = async () => {
  sshConnected.value = true
  showSshModal.value = false
  await checkEnvironment()
  showToast('success', 'SSH 连接成功')
}

const openDashboard = async () => {
  await runAsyncOnce({
    isRunning: () => dashboardOpening.value,
    setRunning: (running) => {
      dashboardOpening.value = running
    },
    action: async () => {
      try {
        await invoke('open_web_ui')
        markActionResult('打开 Dashboard', true, '已静默打开 Dashboard（携带 token）')
      } catch {
        markActionResult('打开 Dashboard', false, '', '打开 Dashboard 失败')
      }
    },
  })
}

const openToolPanel = async (toolId: string) => {
  if (toolId === 'config') {
    navigateTo('ai-config')
    return
  }

  if (toolId === 'channels') {
    navigateTo('channels')
    return
  }

  if (toolId === 'doctor') {
    navigateTo('diagnostics')
    return
  }

  if (toolId === 'install-service') {
    await runWithPendingTool(toolId, async () => {
      try {
        await invoke('install_gateway_service')
        const ready = await waitForGatewayReady(checkGatewayHealth, DEFAULT_GATEWAY_READY_OPTIONS)
        if (!ready) {
          throw new Error('服务安装成功，但网关在预期时间内未对外提供服务')
        }
        markActionResult('安装网关服务', true, '网关服务已安装并完成健康检查')
        await checkEnvironment()
      } catch (error) {
        markActionResult('安装网关服务', false, '', `安装失败: ${String(error)}`)
      }
    })
    return
  }

  if (toolId === 'webui') {
    await openDashboard()
    return
  }

  if (toolId === 'restart') {
    // 立即显示提示
    showToast('info', '正在重启网关，请等待...')
    await runWithPendingTool(toolId, async () => {
      try {
        if (currentEnv.value.mode === 'ssh') {
          await invoke('ssh_restart_gateway')
        } else {
          await invoke('restart_gateway')
        }
        const ready = await waitForGatewayReady(checkGatewayHealth, DEFAULT_GATEWAY_READY_OPTIONS)
        if (!ready) {
          throw new Error('重启命令已发送，但网关在预期时间内未恢复可访问')
        }
        markActionResult('重启网关', true, '网关已重启并完成健康检查')
        await checkEnvironment()
      } catch (error) {
        markActionResult('重启网关', false, '', `重启失败: ${String(error)}`)
      }
    })
    return
  }

  if (toolId === 'tui') {
    try {
      await invoke('open_tui')
      markActionResult('打开 TUI', true, '已打开 TUI')
    } catch {
      markActionResult('打开 TUI', false, '', '打开 TUI 失败')
    }
    return
  }

  if (toolId === 'start') {
    // 立即显示提示
    showToast('info', '正在启动网关，请稍候...')
    await runWithPendingTool(toolId, async () => {
      try {
        if (currentEnv.value.mode === 'ssh') {
          await invoke('ssh_start_gateway')
        } else {
          await invoke('start_gateway')
        }
        const ready = await waitForGatewayReady(checkGatewayHealth, DEFAULT_GATEWAY_READY_OPTIONS)
        if (!ready) {
          throw new Error('启动命令已发送，但网关在预期时间内未进入可访问状态')
        }
        markActionResult('启动网关服务', true, '网关已启动并完成健康检查')
        await checkEnvironment()
      } catch (error) {
        markActionResult('启动网关服务', false, '', `启动失败: ${String(error)}`)
      }
    })
    return
  }

  if (toolId === 'stop') {
    // 立即显示提示
    showToast('info', '正在停止网关，请稍候...')
    await runWithPendingTool(toolId, async () => {
      try {
        if (currentEnv.value.mode === 'ssh') {
          await invoke('ssh_stop_gateway')
        } else {
          await invoke('stop_gateway')
        }
        markActionResult('停止网关服务', true, '停止命令已发送')
        await checkEnvironment()
      } catch (error) {
        markActionResult('停止网关服务', false, '', `停止失败: ${String(error)}`)
      }
    })
    return
  }

  markActionResult('未知动作', false, '', '当前动作暂未开放')
}

const handleInstallComplete = async () => {
  targetMode.value = 'local'
  lastActionFailed.value = false
  if (shouldClearQuickSetupSessionAfterInstall('local')) {
    clearQuickSetupSession()
    quickSetupResumePending.value = false
    quickSetupDebugOpen.value = false
  }
  activeNav.value = 'overview'
  await checkEnvironment()
}

const handleQuickSetupComplete = async () => {
  targetMode.value = 'local'
  lastActionFailed.value = false
  quickSetupDebugOpen.value = false
  quickSetupResumePending.value = false
  activeNav.value = 'overview'
  await checkEnvironment()
}

const handleQuickSetupClose = () => {
  quickSetupDebugOpen.value = false
  quickSetupResumePending.value = false
}

onMounted(async () => {
  loadThemeMode()
  document.addEventListener('click', handleGlobalClick)
  // 异步执行环境检测，不阻塞初始渲染
  loadSshProfiles().catch(console.error)
  checkEnvironment().catch(console.error)
})

onUnmounted(() => {
  document.removeEventListener('click', handleGlobalClick)
  if (toastTimer) clearTimeout(toastTimer)
})
</script>


<template>
  <div class="oc-app-shell dashboard-shell">
    <div class="dashboard-container">
      <!-- 顶部命令栏 -->
      <TopBar
        :active-nav="activeNav"
        :theme-mode="themeMode"
        @navigate="navigateTo"
        @theme-change="applyThemeMode"
      />

      <!-- 主内容区域 -->
      <div class="dashboard-main">
        <!-- 仪表盘矩阵（仅在overview页面且非门禁状态时显示） -->
        <DashboardGrid
          v-if="!isGateActive && !quickSetupForcedOpen && activeNav === 'overview'"
          :active-nav="activeNav"
          :env-status="envStatus"
          :gateway-reachable="gatewayReachable"
          :gateway-checking="gatewayChecking"
          :config-loaded="configLoaded"
          :config-file-path="configFilePath"
          :primary-model-valid="primaryModelValid"
          :openclaw-installed="openclawInstalled"
          :gateway-service-installed="gatewayServiceInstalled"
          :theme-mode="themeMode"
          :env-mode="currentEnv.mode"
          :ssh-connected="sshConnected"
          :bindings-count="bindingsCount"
          :active-bindings-count="activeBindingsCount"
          :channels-enabled="channelsEnabled"
          :total-channels="totalChannels"
          :presets-count="presetsCount"
          :active-presets-count="activePresetsCount"
          :workspaces-count="workspacesCount"
          :active-workspaces-count="activeWorkspacesCount"
          :total-jobs="totalJobs"
          :enabled-jobs="enabledJobs"
          :jobs-with-errors="jobsWithErrors"
          @navigate="navigateTo"
          @action="handleCardAction"
        />

        <!-- 门禁状态内容 -->
        <div v-else-if="isGateActive || quickSetupForcedOpen" class="dashboard-content">
          <div class="oc-main-scroll-page">
            <!-- 环境选择界面 -->
            <div
              v-if="isGateActive && gateState === 'NO_TARGET'"
              class="oc-panel p-6"
              style="min-height: 400px; display: flex; flex-direction: column; justify-content: center; align-items: center;"
            >
              <div style="text-align: center; max-width: 500px;">
                <h3 class="text-2xl font-semibold mb-4" style="color: var(--oc-text-primary);">
                  🎯 欢迎使用 Clawlite
                </h3>
                <p class="text-base mb-8" style="color: var(--oc-text-secondary);">
                  首次使用需要选择运行环境。请选择本地或 SSH 作为唯一运行环境入口。
                </p>

                <div class="grid gap-4" style="grid-template-columns: repeat(2, 1fr); width: 100%;">
                  <button
                    type="button"
                    class="rounded-xl border p-6 text-left transition-all hover:scale-105 hover:shadow-lg"
                    :style="{
                      borderColor: targetMode === 'local' ? 'var(--primary-400)' : 'var(--oc-card-border)',
                      background: targetMode === 'local' ? 'var(--primary-50)' : 'var(--oc-card-elevated)'
                    }"
                    @click="chooseLocalTarget"
                  >
                    <div class="flex items-center gap-3 mb-3">
                      <div class="w-10 h-10 rounded-full flex items-center justify-center" style="background: var(--primary-100);">
                        💻
                      </div>
                      <p class="text-lg font-semibold" style="color: var(--oc-text-primary);">安装在本地</p>
                    </div>
                    <p class="text-sm" style="color: var(--oc-text-muted);">使用本机环境安装并运行 OpenClaw</p>
                  </button>

                  <button
                    type="button"
                    class="rounded-xl border p-6 text-left transition-all hover:scale-105 hover:shadow-lg"
                    :style="{
                      borderColor: targetMode === 'ssh' ? 'var(--primary-400)' : 'var(--oc-card-border)',
                      background: targetMode === 'ssh' ? 'var(--primary-50)' : 'var(--oc-card-elevated)'
                    }"
                    @click="chooseSshTarget"
                  >
                    <div class="flex items-center gap-3 mb-3">
                      <div class="w-10 h-10 rounded-full flex items-center justify-center" style="background: var(--primary-100);">
                        🔌
                      </div>
                      <p class="text-lg font-semibold" style="color: var(--oc-text-primary);">连接 SSH 环境</p>
                    </div>
                    <p class="text-sm" style="color: var(--oc-text-muted);">后续所有操作基于该 SSH 环境执行</p>
                  </button>
                </div>
              </div>
            </div>

            <!-- 安装/配置界面 -->
            <div
              v-else-if="isGateActive && (gateState === 'NEED_INSTALL' || gateState === 'NEED_CONFIG')"
              class="oc-panel p-6"
            >
              <h3 class="text-xl font-semibold" style="color: var(--oc-text-primary);">
                {{
                  gateState === 'NEED_INSTALL'
                    ? '完成安装'
                    : '完成模型配置'
                }}
              </h3>
              <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
                {{
                  gateState === 'NEED_INSTALL'
                    ? '当前环境已确定，请继续完成安装。'
                    : '环境已安装，下一步是完成配置并验证生效。'
                }}
              </p>

              <div v-if="gateState === 'NO_TARGET'" class="mt-4 grid gap-3 md:grid-cols-2">
                <button
                  type="button"
                  class="rounded-[12px] border p-4 text-left transition-colors"
                  :style="{
                    borderColor: targetMode === 'local' ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                    background: targetMode === 'local' ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)'
                  }"
                  @click="chooseLocalTarget"
                >
                  <p class="text-base font-semibold" style="color: var(--oc-text-primary);">安装在本地</p>
                  <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">使用本机环境安装并运行 OpenClaw。</p>
                </button>

                <button
                  type="button"
                  class="rounded-[12px] border p-4 text-left transition-colors"
                  :style="{
                    borderColor: targetMode === 'ssh' ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                    background: targetMode === 'ssh' ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)'
                  }"
                  @click="chooseSshTarget"
                >
                  <p class="text-base font-semibold" style="color: var(--oc-text-primary);">连接 SSH 环境</p>
                  <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">后续所有操作基于该 SSH 环境执行。</p>
                </button>
              </div>

              <div v-else-if="gateState === 'NEED_INSTALL' && targetMode === 'ssh'" class="mt-4 space-y-3">
                <div class="rounded-[12px] border px-3 py-2 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);">
                  当前环境：<strong style="color: var(--oc-text-primary);">{{ targetMode === 'ssh' ? 'SSH' : '本地' }}</strong>
                </div>

                <div v-if="targetMode === 'ssh'" class="rounded-[12px] border px-3 py-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);">
                  <p>1. 先通过右上角环境入口建立 SSH 连接</p>
                  <p class="mt-1">2. 在远程执行安装后，点击"重新检测环境"</p>
                  <div class="mt-3 flex flex-wrap gap-2">
                    <button class="oc-toolbar-btn h-9 px-3" type="button" @click="showSshModal = true">
                      连接 SSH
                    </button>
                    <button class="oc-toolbar-btn h-9 px-3" type="button" @click="checkEnvironment">
                      重新检测环境
                    </button>
                  </div>
                </div>

                <div class="flex flex-wrap gap-2">
                  <button class="oc-toolbar-btn h-9 px-3" type="button" @click="chooseLocalTarget">改用本地</button>
                  <button class="oc-toolbar-btn h-9 px-3" type="button" @click="chooseSshTarget">改用 SSH</button>
                </div>
              </div>

              <div v-else class="mt-4 space-y-2">
                <div class="rounded-[12px] border px-3 py-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);">
                  检测结果为"待配置"，请先完成以下任一操作：
                </div>
                <div class="flex flex-wrap gap-2">
                  <button class="oc-toolbar-btn h-10 px-4" type="button" @click="runManualConfig">
                    手动配置
                  </button>
                  <button class="oc-toolbar-btn h-10 px-4" type="button" @click="applyDefaultConfig">
                    使用默认配置
                  </button>
                  <button class="oc-toolbar-btn h-10 px-4" type="button" @click="checkEnvironment">
                    重新检测环境
                  </button>
                </div>
              </div>
            </div>

            <QuickSetupGuide
              v-else-if="shouldShowQuickSetupGuide && envStatus"
              class="h-full"
              :show-toast="showToast"
              :show-close-action="shouldShowQuickSetupCloseAction"
              :system-os="envStatus.system.os"
              @close="handleQuickSetupClose"
              @complete="handleQuickSetupComplete"
            />

            <InstallPage
              v-else-if="gateState === 'NEED_INSTALL' && targetMode === 'local'"
              class="h-full"
              :mode="'local'"
              :env-connected="true"
              :openclaw-installed="openclawInstalled"
              @install-complete="handleInstallComplete"
            />
          </div>
        </div>

        <!-- 正常页面导航内容 -->
        <div v-else class="dashboard-content">
          <div class="oc-main-scroll-page">
            <StatusDashboard
              v-if="activeNav === 'overview' && envStatus"
              class="oc-page-root"
              :env-status="envStatus"
              :gateway-reachable="gatewayReachable"
              :env-mode="currentEnv.mode"
              :is-windows="isWindows"
              :gateway-service-installed="gatewayServiceInstalled"
              :pending-tool-id="pendingToolId"
              @open-tool="openToolPanel"
            />

            <div v-else-if="activeNav === 'ai-config'" class="oc-page-root">
              <ConfigPage
                v-if="envStatus && openclawInstalled"
                class="oc-page-root"
                :show-toast="showToast"
                :env-mode="currentEnv.mode"
                :env-ssh-connected="sshConnected"
              />
              <div v-else class="oc-panel p-6">
                <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">模型配置不可用</h3>
                <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">请先在"安装与接入"中完成安装。</p>
              </div>
            </div>

            <div v-else-if="activeNav === 'bindings'" class="oc-page-root">
              <BindingsPage
                v-if="envStatus && openclawInstalled"
                class="oc-page-root"
                :show-toast="showToast"
                :env-mode="currentEnv.mode"
                :env-ssh-connected="sshConnected"
              />
              <div v-else class="oc-panel p-6">
                <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">绑定管理不可用</h3>
                <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">请先在"安装与接入"中完成安装。</p>
              </div>
            </div>

            <div v-else-if="activeNav === 'ai-assistant'" class="oc-page-root ai-assistant-page">
              <ChatArea />
            </div>

            <DiagnosticsPage
              v-else-if="activeNav === 'diagnostics'"
              class="oc-page-root"
              :app-state="appState === 'ERROR' || appState === 'DEGRADED' ? appState : 'READY'"
              :env-mode="currentEnv.mode"
              :openclaw-installed="openclawInstalled"
              @refresh="checkEnvironment"
            />

            <div v-else-if="activeNav === 'channels'" class="oc-page-root">
              <MessageChannelsPage class="h-full min-h-0" :show-toast="showToast" :system-os="currentSystemOs" />
            </div>

            <div v-else-if="activeNav === 'skill-presets'" class="oc-page-root">
              <SkillPresetsPage class="h-full min-h-0" />
            </div>

            <div v-else-if="activeNav === 'agent-workspaces'" class="oc-page-root">
              <AgentWorkspacesPage class="h-full min-h-0" />
            </div>

            <div v-else-if="activeNav === 'cron-jobs'" class="oc-page-root">
              <CronJobsPage class="h-full min-h-0" :show-toast="showToast" />
            </div>

            <div v-else class="space-y-3">
              <section class="oc-panel p-6">
                <h3 class="text-xl font-semibold" style="color: var(--oc-text-primary);">系统设置</h3>
                <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">连接相关操作统一从顶部环境入口管理，设置页仅保留偏好项。</p>
              </section>

              <section class="oc-panel p-6">
                <div class="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <h4 class="text-lg font-semibold" style="color: var(--oc-text-primary);">工具设置</h4>
                    <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
                      按当前环境修改配置文件，不覆盖你已有的其它字段。
                    </p>
                  </div>
                  <span class="rounded-[10px] border px-2.5 py-1 text-xs" style="border-color: var(--oc-card-border); color: var(--oc-text-secondary);">
                    {{ currentEnv.mode === 'ssh' ? 'SSH 环境' : '本地环境' }}
                  </span>
                </div>

                <div class="mt-4 rounded-[12px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                  <div class="flex flex-wrap items-center justify-between gap-3">
                    <div>
                      <p class="text-sm font-medium" style="color: var(--oc-text-primary);">浏览器默认 Profile</p>
                      <p class="mt-1 text-xs" style="color: var(--oc-text-muted);">
                        开启时写入 <code>browser.defaultProfile</code> = <code>"openclaw"</code>；关闭时仅删除 <code>defaultProfile</code>，保留 <code>browser</code> 及其它设置。
                      </p>
                    </div>
                    <div class="inline-flex items-center gap-3">
                      <button
                        type="button"
                        aria-label="toggle-browser-default-profile"
                        class="relative inline-flex h-6 w-11 items-center rounded-full border transition-colors"
                        :style="{
                          borderColor: browserDefaultProfileEnabled ? 'color-mix(in srgb, var(--oc-success) 55%, transparent)' : 'var(--oc-card-border)',
                          background: browserDefaultProfileEnabled
                            ? 'color-mix(in srgb, var(--oc-success) 28%, transparent)'
                            : 'color-mix(in srgb, var(--oc-card-elevated) 92%, transparent)'
                        }"
                        :disabled="browserSettingSwitchDisabled"
                        @click="toggleBrowserDefaultProfile"
                      >
                        <span
                          class="h-4 w-4 rounded-full border transition-transform"
                          :style="{
                            borderColor: 'var(--oc-card-border)',
                            background: 'var(--oc-card)',
                            transform: browserDefaultProfileEnabled ? 'translateX(22px)' : 'translateX(2px)'
                          }"
                        />
                      </button>
                      <span class="text-xs" :style="{ color: browserDefaultProfileEnabled ? 'var(--oc-success)' : 'var(--oc-text-muted)' }">
                        {{ browserSettingStatusText }}
                      </span>
                    </div>
                  </div>
                </div>

                <p v-if="browserSettingPath" class="mt-3 text-xs" style="color: var(--oc-text-muted);">
                  配置文件：{{ browserSettingPath }}
                </p>
                <p v-if="browserSettingError" class="mt-2 text-xs" style="color: var(--oc-danger);">
                  {{ browserSettingError }}
                </p>
              </section>

              <section class="oc-panel p-6">
                <div class="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <h4 class="text-lg font-semibold" style="color: var(--oc-text-primary);">页面调试</h4>
                    <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
                      从设置页直接打开快速引导，便于调试布局、主题色和页面内容。
                    </p>
                  </div>
                  <span
                    class="rounded-[10px] border px-2.5 py-1 text-xs"
                    style="border-color: color-mix(in srgb, var(--oc-accent) 12%, var(--oc-card-border)); color: var(--oc-accent);"
                  >
                    调试入口
                  </span>
                </div>

                <div
                  class="mt-4 rounded-[12px] border p-4"
                  style="border-color: color-mix(in srgb, var(--oc-accent) 10%, var(--oc-card-border)); background: color-mix(in srgb, var(--oc-accent-soft) 18%, var(--oc-card) 82%);"
                >
                  <div class="flex flex-wrap items-center justify-between gap-4">
                    <div>
                      <p class="text-sm font-medium" style="color: var(--oc-text-primary);">打开快速引导页面</p>
                      <p class="mt-1 text-xs leading-6" style="color: var(--oc-text-secondary);">
                        使用当前本地环境数据渲染快速引导，用于检查满高布局与细节视觉效果。
                      </p>
                    </div>

                    <Button variant="default" @click="openQuickSetupDebug">
                      打开快速引导
                    </Button>
                  </div>
                </div>
              </section>

              <section v-if="showOpenClawUninstallAction" class="oc-panel p-6">
                <div class="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <h4 class="text-lg font-semibold" style="color: var(--oc-text-primary);">危险操作</h4>
                    <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
                      卸载本机 OpenClaw 全局 npm 包，并移除后台网关服务。
                    </p>
                  </div>
                  <span
                    class="rounded-[10px] border px-2.5 py-1 text-xs"
                    style="border-color: color-mix(in srgb, var(--oc-danger) 24%, var(--oc-card-border)); color: var(--oc-danger);"
                  >
                    仅本地环境
                  </span>
                </div>

                <div
                  class="mt-4 rounded-[12px] border p-4"
                  style="border-color: color-mix(in srgb, var(--oc-danger) 24%, var(--oc-card-border)); background: color-mix(in srgb, var(--oc-danger) 6%, var(--oc-card));"
                >
                  <div class="flex flex-wrap items-start justify-between gap-4">
                    <div class="max-w-2xl">
                      <p class="text-sm font-medium" style="color: var(--oc-text-primary);">卸载 OpenClaw</p>
                      <p class="mt-1 text-xs leading-6" style="color: var(--oc-text-muted);">
                        会删除全局 <code>openclaw</code> npm 包并卸载网关后台服务。Windows 下也会尝试卸载通过
                        <code>nssm</code> 安装的 <code>openclaw-gateway</code> 服务；最后一步可选择是否删除
                        <code>~/.openclaw</code>。
                      </p>
                    </div>

                    <Button
                      variant="destructive"
                      :disabled="openClawUninstallActionState.disabled"
                      :title="openClawUninstallActionState.reason || '卸载 OpenClaw'"
                      @click="openOpenClawUninstallFlow"
                    >
                      卸载 OpenClaw
                    </Button>
                  </div>
                </div>

                <p
                  v-if="openClawUninstallActionState.reason"
                  class="mt-3 text-xs"
                  style="color: var(--oc-text-muted);"
                >
                  {{ openClawUninstallActionState.reason }}
                </p>
              </section>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部 Dock 栏 -->
      <DockBar
        :active-nav="activeNav"
        @navigate="navigateTo"
      />
    </div>

    <!-- SSH 连接模态框 -->
    <SshConnectModal
      v-if="showSshModal"
      @close="showSshModal = false"
      @connected="handleSshConnected"
      @fingerprint="handleFingerprint"
    />

    <SshFingerprintDialog
      v-if="showFingerprintDialog && sshFingerprint"
      :fingerprint="sshFingerprint"
      @confirm="confirmFingerprint"
      @reject="rejectFingerprint"
    />

    <div
      v-if="uninstallOpenClawStep === 'confirm'"
      class="oc-modal-overlay"
      @click.self="closeOpenClawUninstallFlow"
    >
      <Card class="oc-modal-card w-full max-w-lg p-6">
        <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">确认卸载 OpenClaw</h3>
        <p class="mt-1 text-sm leading-6" style="color: var(--oc-text-muted);">
          卸载会按安装流程反向清理当前用户下的 OpenClaw 组件；本步骤默认保留 <code>~/.openclaw</code>，下一步可选是否一并删除配置目录。
        </p>

        <ul class="mt-4 space-y-2 text-sm leading-6" style="color: var(--oc-text-secondary);">
          <li v-for="item in uninstallCleanupItemsWithoutConfig" :key="item" class="flex gap-2">
            <span style="color: var(--oc-danger);">•</span>
            <span>{{ item }}</span>
          </li>
        </ul>

        <div class="mt-5 flex justify-end gap-2">
          <Button variant="outline" :disabled="uninstallOpenClawLoading" @click="closeOpenClawUninstallFlow">
            取消
          </Button>
          <Button variant="destructive" :disabled="uninstallOpenClawLoading" @click="continueOpenClawUninstallFlow">
            继续卸载
          </Button>
        </div>
      </Card>
    </div>

    <div
      v-if="uninstallOpenClawStep === 'phrase'"
      class="oc-modal-overlay"
      @click.self="closeOpenClawUninstallFlow"
    >
      <Card class="oc-modal-card w-full max-w-lg p-6">
        <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">输入确认短语</h3>
        <p class="mt-1 text-sm leading-6" style="color: var(--oc-text-muted);">
          请输入下面的确认短语后继续卸载。
        </p>

        <div class="mt-4 rounded-[14px] border p-3" style="border-color: color-mix(in srgb, var(--oc-danger) 28%, transparent); background: color-mix(in srgb, var(--oc-danger) 8%, transparent);">
          <div class="flex flex-wrap items-center gap-2">
            <button
              type="button"
              class="rounded-[10px] px-3 py-2 text-sm font-bold transition-opacity hover:opacity-85"
              style="background: color-mix(in srgb, var(--oc-danger) 14%, transparent); color: var(--oc-danger);"
              :disabled="uninstallOpenClawLoading"
              @click="copyOpenClawUninstallPhrase"
            >
              {{ OPENCLAW_UNINSTALL_CONFIRM_PHRASE }}
            </button>
            <Button variant="outline" size="sm" :disabled="uninstallOpenClawLoading" @click="copyOpenClawUninstallPhrase">
              点击复制
            </Button>
          </div>
        </div>

        <div class="mt-4">
          <Input
            :model-value="uninstallOpenClawInput"
            :placeholder="OPENCLAW_UNINSTALL_CONFIRM_PHRASE"
            :disabled="uninstallOpenClawLoading"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            @update:model-value="uninstallOpenClawInput = String($event)"
          />
        </div>

        <p
          class="mt-2 text-xs"
          :style="{ color: uninstallOpenClawInput && !uninstallOpenClawPhraseValid ? 'var(--oc-danger)' : 'var(--oc-text-quiet)' }"
        >
          {{
            uninstallOpenClawInput && !uninstallOpenClawPhraseValid
              ? '确认短语不匹配，请完整输入。'
              : `请完整输入：${OPENCLAW_UNINSTALL_CONFIRM_PHRASE}`
          }}
        </p>

        <div class="mt-5 flex justify-end gap-2">
          <Button variant="outline" :disabled="uninstallOpenClawLoading" @click="closeOpenClawUninstallFlow">
            取消
          </Button>
          <Button
            variant="destructive"
            :disabled="uninstallOpenClawLoading || !uninstallOpenClawPhraseValid"
            @click="confirmOpenClawUninstallPhraseStep"
          >
            继续
          </Button>
        </div>
      </Card>
    </div>

    <div
      v-if="uninstallOpenClawStep === 'config'"
      class="oc-modal-overlay"
      @click.self="closeOpenClawUninstallFlow"
    >
      <Card class="oc-modal-card w-full max-w-lg p-6">
        <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">是否删除 ~/.openclaw</h3>
        <p class="mt-1 text-sm leading-6" style="color: var(--oc-text-muted);">
          如果一并删除，将把本地配置、工作区、缓存、日志和托管运行时一起清理，同时回收为 OpenClaw 写入的用户环境配置。
        </p>

        <ul class="mt-4 space-y-2 text-sm leading-6" style="color: var(--oc-text-secondary);">
          <li v-for="item in uninstallCleanupItemsWithConfig" :key="item" class="flex gap-2">
            <span style="color: var(--oc-danger);">•</span>
            <span>{{ item }}</span>
          </li>
        </ul>

        <div class="mt-5 flex flex-wrap justify-end gap-2">
          <Button variant="outline" :disabled="uninstallOpenClawLoading" @click="closeOpenClawUninstallFlow">
            取消
          </Button>
          <Button variant="outline" :disabled="uninstallOpenClawLoading" @click="runOpenClawUninstall(false)">
            仅卸载，不删配置
          </Button>
          <Button variant="destructive" :disabled="uninstallOpenClawLoading" @click="runOpenClawUninstall(true)">
            删除配置并卸载
          </Button>
        </div>
      </Card>
    </div>
    <Toast v-if="toast" :type="toast.type" :message="toast.message" @close="closeToast" />

    <div v-if="loading" class="fixed inset-0 z-[110] flex items-center justify-center bg-black/30 backdrop-blur-[1px]">
      <div class="flex items-center gap-3 rounded-xl border px-4 py-3" style="border-color: var(--oc-card-border); background: var(--oc-card); box-shadow: var(--oc-shadow-popover);">
        <div class="h-5 w-5 animate-spin rounded-full border-2 border-[var(--oc-accent)] border-t-transparent" />
        <span class="text-sm" style="color: var(--oc-text-primary);">{{ loadingMessage }}</span>
      </div>
    </div>
  </div>
</template>
