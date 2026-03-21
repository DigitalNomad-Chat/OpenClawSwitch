<script setup lang="ts">
import { ref, computed } from 'vue'
import { Search, Sun, Moon, Monitor, ChevronDown, User } from 'lucide-vue-next'

interface Props {
  activeNav?: string
  themeMode?: 'system' | 'light' | 'dark'
}

const props = withDefaults(defineProps<Props>(), {
  activeNav: 'overview',
  themeMode: 'system'
})

const emit = defineEmits<{
  navigate: [id: string]
  themeChange: [mode: 'system' | 'light' | 'dark']
}>()

// 命令面板状态
const showCommandPalette = ref(false)

// 当前位置标签
const currentPageLabel = computed(() => {
  const labels: Record<string, string> = {
    overview: '服务状态',
    'ai-config': '配置管理',
    bindings: '绑定管理',
    diagnostics: '诊断工具',
    channels: '消息渠道',
    'skill-presets': '技能预设',
    settings: '系统设置'
  }
  return labels[props.activeNav] || '仪表盘'
})

// 主题图标
const themeIcon = computed(() => {
  if (props.themeMode === 'light') return Sun
  if (props.themeMode === 'dark') return Moon
  return Monitor
})

// 快捷键提示显示状态
const showShortcutHint = ref(false)

// 切换主题
const cycleTheme = () => {
  const modes: Array<'system' | 'light' | 'dark'> = ['system', 'light', 'dark']
  const currentIndex = modes.indexOf(props.themeMode)
  const nextIndex = (currentIndex + 1) % modes.length
  emit('themeChange', modes[nextIndex])
}

// 打开命令面板
const openCommandPalette = () => {
  showCommandPalette.value = true
  emit('navigate', 'command-palette')
}
</script>

<template>
  <div class="top-bar">
    <!-- 左侧：Logo 和当前位置 -->
    <div class="top-bar-left">
      <div class="app-logo">
        <img
          src="../../assets/app-icon.png"
          alt="Clawlite"
          class="logo-icon"
        />
        <span class="app-name">Clawlite</span>
      </div>

      <div class="page-indicator">
        <span class="page-label">{{ currentPageLabel }}</span>
      </div>
    </div>

    <!-- 右侧：搜索和操作 -->
    <div class="top-bar-right">
      <!-- 命令搜索按钮 -->
      <button
        class="top-bar-btn command-btn"
        @click="openCommandPalette"
        title="搜索功能 (⌘K)"
      >
        <Search class="btn-icon" />
        <span class="btn-text">搜索</span>
        <span class="btn-shortcut">⌘K</span>
      </button>

      <!-- 主题切换按钮 -->
      <button
        class="top-bar-btn"
        @click="cycleTheme"
        :title="`主题：${themeMode === 'system' ? '跟随系统' : themeMode === 'light' ? '浅色' : '深色'} (点击切换)`"
      >
        <component :is="themeIcon" class="btn-icon-only" />
      </button>

      <!-- 用户菜单 -->
      <button
        class="top-bar-btn user-btn"
        title="用户菜单"
      >
        <User class="btn-icon-only" />
      </button>
    </div>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════
   顶部命令栏 - Top Bar
   ═══════════════════════════════════════════════════════════ */

.top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-4);
  padding: var(--spacing-4) var(--spacing-6);
  background: var(--bg-surface-elevated);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border-bottom: 1px solid var(--oc-divider-soft);
  position: relative;
}

/* 顶部装饰线 */
.top-bar::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg,
    transparent 0%,
    var(--primary-200) 50%,
    transparent 100%
  );
  opacity: 0.3;
}

/* ═══════════════════════════════════════════════════════════
   左侧区域 - Left Section
   ═══════════════════════════════════════════════════════════ */

.top-bar-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-6);
}

.app-logo {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
}

.logo-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--oc-divider-soft);
}

.app-name {
  font-family: var(--font-display);
  font-size: var(--text-base);
  font-weight: var(--font-weight-bold);
  color: var(--oc-text-primary);
  letter-spacing: -0.02em;
}

.page-indicator {
  display: flex;
  align-items: center;
  padding: var(--spacing-2) var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-full);
}

.page-label {
  font-size: var(--text-sm);
  color: var(--oc-text-secondary);
  font-weight: var(--font-weight-medium);
}

/* ═══════════════════════════════════════════════════════════
   右侧区域 - Right Section
   ═══════════════════════════════════════════════════════════ */

.top-bar-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
}

/* ═══════════════════════════════════════════════════════════
   顶部按钮 - Top Bar Buttons
   ═══════════════════════════════════════════════════════════ */

.top-bar-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-2) var(--spacing-3);
  border-radius: var(--radius-lg);
  border: 1px solid transparent;
  background: transparent;
  color: var(--oc-text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: all var(--duration-200) var(--ease-smooth);
}

.top-bar-btn:hover {
  background: var(--bg-secondary);
  border-color: var(--oc-divider-soft);
  color: var(--oc-text-primary);
  transform: translateY(-1px);
}

.top-bar-btn:focus-visible {
  outline: none;
  border-color: var(--primary-400);
  box-shadow: 0 0 0 3px rgba(20, 184, 166, 0.2);
}

/* 命令按钮特殊样式 */
.command-btn {
  background: var(--bg-secondary);
  border-color: var(--oc-divider-soft);
  min-width: 140px;
}

.command-btn:hover {
  border-color: var(--primary-300);
  background: var(--bg-primary);
}

.btn-icon {
  width: 18px;
  height: 18px;
}

.btn-icon-only {
  width: 18px;
  height: 18px;
}

.btn-text {
  font-size: var(--text-sm);
}

.btn-shortcut {
  margin-left: auto;
  padding: var(--spacing-1) var(--spacing-2);
  background: var(--oc-divider);
  border-radius: var(--radius-md);
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  color: var(--oc-text-quiet);
}

/* ═══════════════════════════════════════════════════════════
   响应式 - Responsive
   ═══════════════════════════════════════════════════════════ */

@media (max-width: 768px) {
  .top-bar {
    padding: var(--spacing-3) var(--spacing-4);
  }

  .top-bar-left {
    gap: var(--spacing-4);
  }

  .app-name {
    display: none;
  }

  .page-indicator {
    display: none;
  }

  .btn-text {
    display: none;
  }

  .command-btn {
    min-width: auto;
    padding: var(--spacing-2);
  }

  .btn-shortcut {
    display: none;
  }
}
</style>
