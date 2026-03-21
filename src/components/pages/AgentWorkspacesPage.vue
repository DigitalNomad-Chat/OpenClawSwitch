<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { Search, RefreshCw, Star, X } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import type { AgentWorkspaceWithStatus, WorkspaceCategoryInfo } from '@/types/agent-workspace'

// ============================================================================
// 状态定义
// ============================================================================

const workspaceLoading = ref(false)
const agentWorkspaces = ref<AgentWorkspaceWithStatus[]>([])
const workspaceCategories = ref<WorkspaceCategoryInfo[]>([])

// 筛选状态
const workspaceSearchQuery = ref('')
const selectedWorkspaceCategory = ref<string | null>(null)
const showWorkspaceRecommendedOnly = ref(false)

// 部署状态缓存
const deployedWorkspaces = ref<Set<string>>(new Set())

// ============================================================================
// 计算属性
// ============================================================================

/** 过滤后的 Workspace 列表 */
const filteredWorkspaces = computed(() => {
  let result = agentWorkspaces.value

  // 按分类筛选
  if (selectedWorkspaceCategory.value) {
    result = result.filter(w => w.preset.category === selectedWorkspaceCategory.value)
  }

  // 按推荐筛选
  if (showWorkspaceRecommendedOnly.value) {
    result = result.filter(w => w.preset.recommended)
  }

  // 按搜索关键词筛选
  if (workspaceSearchQuery.value.trim()) {
    const query = workspaceSearchQuery.value.toLowerCase()
    result = result.filter(w =>
      w.preset.name.toLowerCase().includes(query) ||
      w.preset.description.toLowerCase().includes(query) ||
      w.preset.tags.some(tag => tag.toLowerCase().includes(query))
    )
  }

  return result
})

/** 收藏的 Workspace 数量 */
const activatedWorkspaceCount = computed(() => {
  return agentWorkspaces.value.filter(w => w.activated).length
})

/** 检查 Workspace 是否已部署 */
const isWorkspaceDeployed = (workspaceId: string) => {
  return deployedWorkspaces.value.has(workspaceId)
}

// ============================================================================
// 生命周期
// ============================================================================

onMounted(async () => {
  await loadData()
})

// ============================================================================
// 数据加载
// ============================================================================

async function loadData() {
  workspaceLoading.value = true
  try {
    const [workspacesData, workspaceCategoriesData] = await Promise.all([
      invoke<AgentWorkspaceWithStatus[]>('get_agent_workspaces_with_status'),
      invoke<Array<{ id: string; icon: string; name: string; description: string; count: number }>>('get_agent_workspace_categories')
    ])

    agentWorkspaces.value = workspacesData
    workspaceCategories.value = workspaceCategoriesData as WorkspaceCategoryInfo[]

    // 检查所有 Workspace 的部署状态
    const deployedSet = new Set<string>()
    for (const workspace of workspacesData) {
      try {
        const deployed = await invoke<boolean>('check_agent_deployed', { workspaceId: workspace.preset.id })
        if (deployed) {
          deployedSet.add(workspace.preset.id)
        }
      } catch (error) {
        console.error(`检查 ${workspace.preset.id} 部署状态失败:`, error)
      }
    }
    deployedWorkspaces.value = deployedSet
  } catch (error) {
    console.error('加载 Agent Workspaces 失败:', error)
  } finally {
    workspaceLoading.value = false
  }
}

// ============================================================================
// 操作处理
// ============================================================================

/** 收藏/取消收藏 Workspace */
async function toggleWorkspaceFavorite(workspaceId: string, activated: boolean) {
  try {
    if (activated) {
      // 如果已部署，先卸载
      const deployed = await invoke<boolean>('check_agent_deployed', { workspaceId })
      if (deployed) {
        const confirmed = confirm('该 Agent 已部署到 OpenClaw，取消收藏将同时卸载。是否继续？')
        if (!confirmed) return

        await invoke('undeploy_agent_workspace', { workspaceId })
      }

      await invoke('deactivate_agent_workspace', { workspaceId })
    } else {
      await invoke('activate_agent_workspace', { workspaceId })
    }
    await loadData()
  } catch (error) {
    console.error('操作失败:', error)
    alert(activated ? '取消收藏失败' : '收藏失败: ' + error)
  }
}

/** 使用 Workspace（部署到 OpenClaw）*/
async function useWorkspace(workspaceId: string) {
  try {
    // 检查是否已部署
    const deployed = await invoke<boolean>('check_agent_deployed', { workspaceId })

    if (deployed) {
      // 已部署，记录使用即可
      await invoke('record_agent_workspace_usage', { workspaceId })
      alert(`该 Agent 已在 OpenClaw 中就绪`)
    } else {
      // 未部署，执行部署
      const result = await invoke<string>('deploy_agent_workspace', { workspaceId })
      alert(result)

      // 部署成功后记录使用
      await invoke('record_agent_workspace_usage', { workspaceId })
    }

    await loadData()
  } catch (error) {
    console.error('操作失败:', error)
    alert('操作失败: ' + error)
  }
}
</script>

<template>
  <div class="agent-workspaces-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-title">
        <h1 style="color: var(--oc-text-primary);">👥 Agent Workspaces</h1>
        <p class="header-subtitle" style="color: var(--oc-text-secondary);">
          管理和使用职场Agent岗位预设
        </p>
      </div>

      <div class="header-actions">
        <div class="activated-badge">
          <Badge v-if="activatedWorkspaceCount > 0" :variant="'success'" :size="'sm'">
            已收藏 {{ activatedWorkspaceCount }} 个岗位
          </Badge>
        </div>
        <Button variant="ghost" size="sm" @click="loadData" :disabled="workspaceLoading">
          <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': workspaceLoading }" />
        </Button>
      </div>
    </div>

    <!-- 搜索和筛选栏 -->
    <div class="filter-bar">
      <!-- 搜索框 -->
      <div class="search-box">
        <Search class="search-icon" />
        <input
          v-model="workspaceSearchQuery"
          type="text"
          placeholder="搜索 Workspace 名称、描述或标签..."
          class="search-input"
        />
        <button v-if="workspaceSearchQuery" @click="workspaceSearchQuery = ''" class="search-clear">
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- 推荐筛选 -->
      <button
        :class="['filter-chip', { active: showWorkspaceRecommendedOnly }]"
        @click="showWorkspaceRecommendedOnly = !showWorkspaceRecommendedOnly"
      >
        <Star class="w-4 h-4" />
        <span>仅推荐</span>
      </button>
    </div>

    <!-- Workspace 分类标签 -->
    <div class="category-tabs">
      <button
        :class="['category-tab', { active: selectedWorkspaceCategory === null }]"
        @click="selectedWorkspaceCategory = null"
      >
        全部 ({{ agentWorkspaces.length }})
      </button>
      <button
        v-for="cat in workspaceCategories"
        :key="cat.id"
        :class="['category-tab', { active: selectedWorkspaceCategory === cat.id }]"
        @click="selectedWorkspaceCategory = cat.id"
      >
        <span>{{ cat.icon }}</span>
        <span>{{ cat.name }}</span>
        <span class="count">({{ cat.count }})</span>
      </button>
    </div>

    <!-- Workspace 列表 -->
    <div v-if="workspaceLoading" class="loading-state">
      <RefreshCw class="animate-spin w-8 h-8" />
      <p>加载中...</p>
    </div>

    <div v-else-if="filteredWorkspaces.length === 0" class="empty-state">
      <p>没有找到匹配的 Workspace</p>
    </div>

    <div v-else class="workspaces-grid">
      <div
        v-for="workspace in filteredWorkspaces"
        :key="workspace.preset.id"
        class="workspace-card"
        :class="{ activated: workspace.activated }"
      >
        <div class="workspace-header">
          <div class="workspace-icon">{{ workspace.preset.icon }}</div>
          <div class="workspace-info">
            <h3 class="workspace-name">{{ workspace.preset.name }}</h3>
            <p class="workspace-description">{{ workspace.preset.description }}</p>
          </div>
          <div class="workspace-status">
            <!-- 部署状态标识 -->
            <span v-if="isWorkspaceDeployed(workspace.preset.id)" class="status-badge deployed">
              🚀 已部署
            </span>
            <span v-else-if="workspace.activated" class="status-badge activated">
              已收藏
            </span>
            <span v-else class="status-badge">未收藏</span>
          </div>
        </div>

        <div class="workspace-body">
          <div class="workspace-section">
            <h4 class="section-title">核心能力</h4>
            <div class="tags">
              <span v-for="cap in workspace.preset.capabilities.slice(0, 3)" :key="cap" class="tag">
                {{ cap }}
              </span>
            </div>
          </div>

          <div class="workspace-section">
            <h4 class="section-title">应用场景</h4>
            <div class="tags">
              <span v-for="scenario in workspace.preset.scenarios.slice(0, 3)" :key="scenario" class="tag">
                {{ scenario }}
              </span>
            </div>
          </div>

          <!-- 技能标签 -->
          <div class="workspace-section">
            <h4 class="section-title">技能标签</h4>
            <div class="tags">
              <span v-for="tag in workspace.preset.tags.slice(0, 4)" :key="tag" class="tag tag-skill">
                {{ tag }}
              </span>
            </div>
          </div>
        </div>

        <div class="workspace-footer">
          <div class="workspace-meta">
            <span v-if="workspace.preset.recommended" class="recommended-badge">
              <Star class="w-3 h-3" />
              推荐
            </span>
            <span v-if="workspace.usageCount && workspace.usageCount > 0" class="usage-count">
              使用 {{ workspace.usageCount }} 次
            </span>
          </div>
          <div class="workspace-actions">
            <!-- 未收藏状态 -->
            <Button
              v-if="!workspace.activated"
              variant="default"
              size="sm"
              @click="toggleWorkspaceFavorite(workspace.preset.id, false)"
            >
              收藏
            </Button>

            <!-- 已收藏状态 -->
            <template v-else>
              <!-- 部署按钮 -->
              <Button
                variant="secondary"
                size="sm"
                @click="useWorkspace(workspace.preset.id)"
              >
                {{ isWorkspaceDeployed(workspace.preset.id) ? '✓ 已部署' : '部署' }}
              </Button>

              <!-- 取消收藏按钮 -->
              <Button
                variant="ghost"
                size="sm"
                @click="toggleWorkspaceFavorite(workspace.preset.id, true)"
              >
                取消收藏
              </Button>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ============================================================================
   页面布局
   ============================================================================ */

.agent-workspaces-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* 页面头部 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 1.5rem 1.5rem 1rem;
  border-bottom: 1px solid var(--oc-divider);
}

.header-title h1 {
  font-size: 1.25rem;
  font-weight: 600;
  margin: 0;
}

.header-subtitle {
  font-size: 0.875rem;
  margin: 0.25rem 0 0;
}

.header-actions {
  display: flex;
  gap: 0.75rem;
  align-items: center;
}

.activated-badge {
  display: flex;
  align-items: center;
}

/* 筛选栏 */
.filter-bar {
  display: flex;
  gap: 0.75rem;
  align-items: center;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--oc-divider);
  flex-wrap: wrap;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 280px;
  max-width: 400px;
}

.search-icon {
  position: absolute;
  left: 0.75rem;
  color: var(--oc-text-tertiary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 0.625rem 2.5rem 0.625rem 2.5rem;
  background: var(--oc-bg-surface);
  border: 1px solid var(--oc-divider);
  border-radius: 0.5rem;
  color: var(--oc-text-primary);
  font-size: 0.875rem;
  outline: none;
  transition: border-color 0.2s;
}

.search-input:focus {
  border-color: var(--oc-accent);
}

.search-clear {
  position: absolute;
  right: 0.5rem;
  padding: 0.25rem;
  background: transparent;
  border: none;
  color: var(--oc-text-tertiary);
  cursor: pointer;
  border-radius: 0.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.search-clear:hover {
  background: var(--oc-bg-hover);
  color: var(--oc-text-secondary);
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.75rem;
  background: var(--oc-bg-surface);
  border: 1px solid var(--oc-divider);
  border-radius: 2rem;
  color: var(--oc-text-secondary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s;
}

.filter-chip:hover {
  background: var(--oc-bg-hover);
  border-color: var(--oc-accent);
}

.filter-chip.active {
  background: color-mix(in srgb, var(--oc-accent) 10%, transparent);
  border-color: var(--oc-accent);
  color: var(--oc-accent);
}

/* 分类标签 */
.category-tabs {
  display: flex;
  gap: 0.5rem;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--oc-divider);
  overflow-x: auto;
  flex-wrap: wrap;
}

.category-tab {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.875rem;
  background: transparent;
  border: none;
  border-radius: 2rem;
  color: var(--oc-text-secondary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.category-tab:hover {
  background: var(--oc-bg-hover);
  color: var(--oc-text-primary);
}

.category-tab.active {
  background: var(--oc-accent);
  color: white;
}

.category-tab .count {
  font-size: 0.75rem;
  opacity: 0.8;
}

/* 加载和空状态 */
.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  color: var(--oc-text-secondary);
}

.loading-state p,
.empty-state p {
  margin-top: 1rem;
  font-size: 0.875rem;
}

/* ============================================================================
   Workspace 卡片网格
   ============================================================================ */

.workspaces-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 1rem;
  padding: 1.5rem;
  overflow-y: auto;
}

.workspace-card {
  background: var(--oc-bg-surface);
  border: 1px solid var(--oc-divider);
  border-radius: 0.75rem;
  padding: 1rem;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
}

.workspace-card:hover {
  border-color: var(--oc-accent);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.workspace-card.activated {
  border-color: var(--oc-success);
  background: color-mix(in srgb, var(--oc-success) 5%, var(--oc-bg-surface));
}

.workspace-header {
  display: flex;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.workspace-icon {
  font-size: 2rem;
  line-height: 1;
  flex-shrink: 0;
}

.workspace-info {
  flex: 1;
  min-width: 0;
}

.workspace-name {
  font-size: 1rem;
  font-weight: 600;
  color: var(--oc-text-primary);
  margin: 0 0 0.25rem;
}

.workspace-description {
  font-size: 0.875rem;
  color: var(--oc-text-secondary);
  margin: 0;
  line-height: 1.4;
}

.workspace-status {
  flex-shrink: 0;
}

.status-badge {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  font-weight: 500;
  background: var(--oc-bg-tertiary);
  color: var(--oc-text-secondary);
  border-radius: 0.25rem;
}

.status-badge.activated {
  background: color-mix(in srgb, var(--oc-success) 20%, transparent);
  color: var(--oc-success);
}

.status-badge.deployed {
  background: color-mix(in srgb, var(--primary-500) 20%, transparent);
  color: var(--primary-600);
  font-weight: var(--font-weight-semibold);
}

.workspace-body {
  flex: 1;
  margin-bottom: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.workspace-section {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.workspace-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--oc-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0;
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.tag {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  background: var(--oc-bg-tertiary);
  color: var(--oc-text-secondary);
  border-radius: 0.25rem;
}

.tag-skill {
  background: color-mix(in srgb, var(--oc-accent) 10%, transparent);
  color: var(--oc-accent);
}

.workspace-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 0.75rem;
  border-top: 1px solid var(--oc-divider);
  margin-top: auto;
}

.workspace-meta {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.recommended-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  color: var(--oc-warning);
  background: color-mix(in srgb, var(--oc-warning) 15%, transparent);
  border-radius: 0.25rem;
}

.usage-count {
  font-size: 0.75rem;
  color: var(--oc-text-tertiary);
}

.workspace-actions {
  display: flex;
  gap: 0.5rem;
}

/* ============================================================================
   响应式
   ============================================================================ */

@media (max-width: 768px) {
  .workspaces-grid {
    grid-template-columns: 1fr;
    padding: 1rem;
  }

  .filter-bar {
    flex-direction: column;
    align-items: stretch;
  }

  .search-box {
    max-width: none;
  }

  .category-tabs {
    padding: 0.75rem 1rem;
  }
}
</style>
