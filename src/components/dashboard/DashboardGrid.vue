<script setup lang="ts">
import { ref, computed, markRaw } from 'vue'
import type { NavPage } from '../../domain/navigation'
import type { EnvironmentStatus } from '../../types/config'

// 导入卡片组件
import StatusCard from './cards/StatusCard.vue'
import ConfigCard from './cards/ConfigCard.vue'
import BindingCard from './cards/BindingCard.vue'
import DiagnosticsCard from './cards/DiagnosticsCard.vue'
import ChannelsCard from './cards/ChannelsCard.vue'
import SettingsCard from './cards/SettingsCard.vue'
import SkillPresetsCard from './cards/SkillPresetsCard.vue'
import AgentWorkspacesCard from './cards/AgentWorkspacesCard.vue'
import CronJobsCard from './cards/CronJobsCard.vue'

interface DiagnosticResults {
  passed: number
  warnings: number
  errors: number
}

interface Props {
  activeNav: NavPage
  // 环境状态
  envStatus?: EnvironmentStatus | null
  gatewayReachable?: boolean
  gatewayChecking?: boolean
  // 配置状态
  configLoaded?: boolean
  configFilePath?: string
  primaryModelValid?: boolean
  // 安装状态
  openclawInstalled?: boolean
  gatewayServiceInstalled?: boolean
  // 环境信息
  envMode?: 'local' | 'ssh'
  sshConnected?: boolean
  // 主题
  themeMode?: 'system' | 'light' | 'dark'
  // 绑定数据（暂用默认值）
  bindingsCount?: number
  activeBindingsCount?: number
  // 渠道数据（暂用默认值）
  channelsEnabled?: number
  totalChannels?: number
  // 诊断数据（暂用默认值）
  diagnosticResults?: DiagnosticResults
  // Skill 预设数据
  presetsCount?: number
  activePresetsCount?: number
  // Agent 工作空间数据
  workspacesCount?: number
  activeWorkspacesCount?: number
  // Cron 定时任务数据
  totalJobs?: number
  enabledJobs?: number
  jobsWithErrors?: number
}

const props = withDefaults(defineProps<Props>(), {
  envStatus: null,
  gatewayReachable: false,
  gatewayChecking: false,
  configLoaded: false,
  configFilePath: '',
  primaryModelValid: false,
  openclawInstalled: false,
  gatewayServiceInstalled: true,
  envMode: 'local',
  sshConnected: false,
  themeMode: 'system',
  bindingsCount: 0,
  activeBindingsCount: 0,
  channelsEnabled: 0,
  totalChannels: 0,
  diagnosticResults: () => ({ passed: 0, warnings: 0, errors: 0 }),
  presetsCount: 0,
  activePresetsCount: 0,
  workspacesCount: 0,
  activeWorkspacesCount: 0,
  totalJobs: 0,
  enabledJobs: 0,
  jobsWithErrors: 0
})

const emit = defineEmits<{
  navigate: [id: NavPage]
  action: [cardId: NavPage, action: string]
}>()

// 当前激活的卡片
const activeCardId = computed(() => props.activeNav)

// 卡片配置
const dashboardCards = ref([
  {
    id: 'overview' as NavPage,
    title: '服务状态',
    icon: 'Activity',
    component: markRaw(StatusCard),
    alwaysVisible: true
  },
  {
    id: 'ai-config' as NavPage,
    title: '配置管理',
    icon: 'Settings',
    component: markRaw(ConfigCard),
    alwaysVisible: true
  },
  {
    id: 'bindings' as NavPage,
    title: '绑定管理',
    icon: 'Link',
    component: markRaw(BindingCard),
    alwaysVisible: true
  },
  {
    id: 'skill-presets' as NavPage,
    title: '技能预设',
    icon: 'Sparkles',
    component: markRaw(SkillPresetsCard),
    alwaysVisible: true
  },
  {
    id: 'agent-workspaces' as NavPage,
    title: 'Agent工作空间',
    icon: 'Users',
    component: markRaw(AgentWorkspacesCard),
    alwaysVisible: true
  },
  {
    id: 'cron-jobs' as NavPage,
    title: 'Cron定时任务',
    icon: 'Clock',
    component: markRaw(CronJobsCard),
    alwaysVisible: true
  },
  {
    id: 'diagnostics' as NavPage,
    title: '诊断工具',
    icon: 'Stethoscope',
    component: markRaw(DiagnosticsCard),
    alwaysVisible: true
  },
  {
    id: 'channels' as NavPage,
    title: '消息渠道',
    icon: 'MessageSquare',
    component: markRaw(ChannelsCard),
    alwaysVisible: true
  },
  {
    id: 'settings' as NavPage,
    title: '系统设置',
    icon: 'Settings2',
    component: markRaw(SettingsCard),
    alwaysVisible: true
  }
])

// 卡片点击处理
const handleCardClick = (cardId: NavPage) => {
  // 发射事件到父组件切换视图
  emit('navigate', cardId)
}

// 卡片按钮操作处理
const handleCardAction = (cardId: NavPage, action: string) => {
  // 转发卡片操作事件到父组件
  emit('action', cardId, action)
}
</script>

<template>
  <div class="dashboard-container">
    <!-- 顶部装饰线 -->
    <div class="dashboard-header-line"></div>

    <!-- 功能矩阵网格 -->
    <div class="dashboard-grid">
      <div
        v-for="(card, index) in dashboardCards"
        :key="card.id"
        class="dashboard-card-wrapper"
        :style="{ animationDelay: `${index * 50}ms` }"
        @click="handleCardClick(card.id)"
      >
        <component
          :is="card.component"
          :title="card.title"
          :icon="card.icon"
          :active="activeCardId === card.id"
          :env-status="envStatus"
          :gateway-reachable="gatewayReachable"
          :checking="gatewayChecking"
          :config-loaded="configLoaded"
          :config-file-path="configFilePath"
          :primary-model-valid="primaryModelValid"
          :openclaw-installed="openclawInstalled"
          :gateway-service-installed="gatewayServiceInstalled"
          :theme-mode="themeMode"
          :env-mode="envMode"
          :ssh-connected="sshConnected"
          :bindings-count="bindingsCount"
          :active-bindings-count="activeBindingsCount"
          :channels-enabled="channelsEnabled"
          :total-channels="totalChannels"
          :diagnostic-results="diagnosticResults"
          :presets-count="presetsCount"
          :active-presets-count="activePresetsCount"
          :workspaces-count="workspacesCount"
          :active-workspaces-count="activeWorkspacesCount"
          :total-jobs="totalJobs"
          :enabled-jobs="enabledJobs"
          :jobs-with-errors="jobsWithErrors"
          @action="(action) => handleCardAction(card.id, action)"
        >
          <!-- 如果组件有默认插槽内容，显示它 -->
          <template v-if="card.defaultContent">
            {{ card.defaultContent }}
          </template>
        </component>
      </div>
    </div>

    <!-- 底部提示 -->
    <div class="dashboard-footer">
      <p class="dashboard-footer-text">
        💡 提示：点击卡片展开详情，使用快捷键 ⌘1-9 快速切换
      </p>
    </div>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════
   仪表盘容器 - Dashboard Container
   ═══════════════════════════════════════════════════════════ */

.dashboard-container {
  width: 100%;
  height: 100%;
  padding: var(--spacing-6);
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}

/* 顶部装饰线 */
.dashboard-header-line {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg,
    transparent 0%,
    var(--primary-300) 50%,
    transparent 100%
  );
  opacity: 0.5;
}

/* ═══════════════════════════════════════════════════════════
   功能矩阵网格 - Card Grid
   ═══════════════════════════════════════════════════════════ */

.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-6);
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: var(--spacing-2);
}

/* 卡片包装器 - 负责动画 */
.dashboard-card-wrapper {
  animation: cardFadeIn 0.4s cubic-bezier(0.4, 0, 0.2, 1) backwards;
  cursor: pointer;
}

/* 卡片入场动画 */
@keyframes cardFadeIn {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* ═══════════════════════════════════════════════════════════
   响应式设计 - Responsive Design
   ═══════════════════════════════════════════════════════════ */

/* 移动端 - 1 列 */
@media (max-width: 768px) {
  .dashboard-container {
    padding: var(--spacing-4);
  }

  .dashboard-grid {
    grid-template-columns: 1fr;
    gap: var(--spacing-4);
  }
}

/* 平板 - 2 列 */
@media (min-width: 769px) and (max-width: 1200px) {
  .dashboard-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: var(--spacing-5);
  }
}

/* 大屏 - 4 列 */
@media (min-width: 1600px) {
  .dashboard-grid {
    grid-template-columns: repeat(4, 1fr);
  }
}

/* ═══════════════════════════════════════════════════════════
   底部提示 - Footer
   ═══════════════════════════════════════════════════════════ */

.dashboard-footer {
  margin-top: var(--spacing-6);
  padding: var(--spacing-4) var(--spacing-6);
  text-align: center;
  border-top: 1px solid var(--oc-divider-soft);
}

.dashboard-footer-text {
  font-size: var(--text-sm);
  color: var(--oc-text-muted);
  margin: 0;
  line-height: var(--leading-relaxed);
}

/* ═══════════════════════════════════════════════════════════
   滚动条样式 - Scrollbar Styling
   ═══════════════════════════════════════════════════════════ */

.dashboard-grid::-webkit-scrollbar {
  width: 8px;
}

.dashboard-grid::-webkit-scrollbar-track {
  background: transparent;
}

.dashboard-grid::-webkit-scrollbar-thumb {
  background: var(--oc-text-quiet);
  border-radius: 4px;
  transition: background 0.2s ease;
}

.dashboard-grid::-webkit-scrollbar-thumb:hover {
  background: var(--oc-text-muted);
}
</style>
