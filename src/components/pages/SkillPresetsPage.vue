<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { Search, RefreshCw, Star, Download, Trash2, Check, AlertTriangle, ExternalLink, X, ChevronRight, FolderOpen, ChevronDown, ChevronUp } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import type { SkillWithStatus, InstalledSkillInfo, SkillsBySource, SkillDocument } from '@/types/preset'
import { CATEGORY_META, SOURCE_META, SKILL_SOURCE_META } from '@/types/preset'

// ============================================================================
// 状态定义
// ============================================================================

interface CategoryInfo {
  id: string
  icon: string
  name: string
  count: number
}

const loading = ref(false)
const skillsData = ref<SkillsBySource | null>(null)

// 预设技能
const presets = computed(() => skillsData.value?.presets || [])
const presetCount = computed(() => presets.value.length)
const installedPresetCount = computed(() => presets.value.filter(s => s.installed).length)
const recommendedPresets = computed(() => presets.value.filter(s => s.preset.recommended))

// 非预设技能（按来源分组）
const workspaceSkills = computed(() => skillsData.value?.workspaceSkills || [])
const agentSkills = computed(() => skillsData.value?.agentSkills || {})

// 纯非预设技能（与预设无关）
const pureNonPresetSkills = computed(() => {
  const workspace: InstalledSkillInfo[] = []
  const agents: Record<string, InstalledSkillInfo[]> = {}

  // workspace 中纯非预设的
  for (const skill of workspaceSkills.value) {
    if (!skill.installedFromPreset) {
      workspace.push(skill)
    }
  }

  // agent 中纯非预设的
  for (const [agentName, skills] of Object.entries(agentSkills.value)) {
    const filtered = skills.filter(s => !s.installedFromPreset)
    if (filtered.length > 0) {
      agents[agentName] = filtered
    }
  }

  return { workspace, agents }
})

const nonPresetCount = computed(() => {
  return pureNonPresetSkills.value.workspace.length +
    Object.values(pureNonPresetSkills.value.agents).flat().length
})

// 筛选状态
const searchQuery = ref('')
const selectedCategory = ref<string | null>(null)
const selectedSource = ref<'all' | 'preset' | 'non-preset'>('all')
const showInstalledOnly = ref(false)

// 折叠状态
const collapsedGroups = ref<Set<string>>(new Set())

// 详情弹窗状态
const selectedSkill = ref<SkillWithStatus | InstalledSkillInfo | null>(null)
const selectedSkillType = ref<'preset' | 'installed'>('preset')
const detailModalOpen = ref(false)
const skillDocument = ref<SkillDocument | null>(null)

// API Key 弹窗状态
const apiKeyModalOpen = ref(false)
const apiKeyRequirements = ref<Array<{
  skill_id: string
  skill_name: string
  skill_icon: string
  api_key_name: string
  env_var: string
  url?: string
  description?: string
  is_configured: boolean
}>>([])

// 依赖指引弹窗状态
const depGuideModalOpen = ref(false)
const depGuideInfo = ref<{
  skill_id: string
  skill_name: string
  has_missing_deps: boolean
  missing_deps: string[]
  install_commands: string[]
  has_install_config: boolean
} | null>(null)

// 安装/卸载操作中
const installingSkillId = ref<string | null>(null)

// 批量选择状态
const selectedSkillIds = ref<Set<string>>(new Set())
const isBatchInstalling = ref(false)
const batchProgress = ref({ current: 0, total: 0 })
const batchResults = ref<Array<{ skillId: string; skillName: string; success: boolean; message?: string }>>([])

// 批量操作弹窗状态
const batchModalOpen = ref(false)

// 安装确认弹窗状态
const installConfirmModalOpen = ref(false)
const skillToInstall = ref<SkillWithStatus | null>(null)

// 删除确认弹窗状态
const deleteConfirmModalOpen = ref(false)
const skillToDelete = ref<InstalledSkillInfo | null>(null)

// Toast 通知状态
const toast = ref<{
  show: boolean
  message: string
  type: 'success' | 'error' | 'warning'
}>({
  show: false,
  message: '',
  type: 'success'
})

// ============================================================================
// 计算属性
// ============================================================================

/** 过滤后的预设技能 */
const filteredPresets = computed(() => {
  let result = presets.value

  // 按分类筛选
  if (selectedCategory.value) {
    result = result.filter(s => s.preset.category === selectedCategory.value)
  }

  // 按推荐筛选
  if (showInstalledOnly.value) {
    result = result.filter(s => s.installed)
  }

  // 按搜索关键词筛选
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(s =>
      s.preset.name.toLowerCase().includes(query) ||
      s.preset.description.toLowerCase().includes(query) ||
      s.preset.tags.some(tag => tag.toLowerCase().includes(query))
    )
  }

  return result
})

/** 过滤后的纯非预设技能 - workspace */
const filteredPureNonPresetWorkspace = computed(() => {
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    return pureNonPresetSkills.value.workspace.filter(s =>
      s.name.toLowerCase().includes(query) ||
      s.description.toLowerCase().includes(query) ||
      s.id.toLowerCase().includes(query)
    )
  }
  return pureNonPresetSkills.value.workspace
})

/** 过滤后的纯非预设技能 - agent */
const filteredPureNonPresetAgent = computed(() => {
  const result: Record<string, InstalledSkillInfo[]> = {}
  for (const [agentName, skills] of Object.entries(pureNonPresetSkills.value.agents)) {
    if (searchQuery.value.trim()) {
      const query = searchQuery.value.toLowerCase()
      const filtered = skills.filter(s =>
        s.name.toLowerCase().includes(query) ||
        s.description.toLowerCase().includes(query) ||
        s.id.toLowerCase().includes(query)
      )
      if (filtered.length > 0) {
        result[agentName] = filtered
      }
    } else {
      result[agentName] = skills
    }
  }
  return result
})

/** 选中的技能数量 */
const selectedCount = computed(() => selectedSkillIds.value.size)

/** 是否有选中的技能 */
const hasSelection = computed(() => selectedSkillIds.value.size > 0)

/** 可选择的技能（未安装的预设） */
const installableSkills = computed(() => {
  return filteredPresets.value.filter(s => !s.installed)
})

/** 是否全选 */
const isAllSelected = computed(() => {
  const installableIds = installableSkills.value.map(s => s.preset.id)
  if (installableIds.length === 0) return false
  return installableIds.every(id => selectedSkillIds.value.has(id))
})

/** 是否部分选中 */
const isSomeSelected = computed(() => {
  const installableIds = installableSkills.value.map(s => s.preset.id)
  const selectedInstallable = installableIds.filter(id => selectedSkillIds.value.has(id))
  return selectedInstallable.length > 0 && selectedInstallable.length < installableIds.length
})

/** 来源选项 */
const sourceOptions = [
  { id: 'all', label: '全部', icon: '📦' },
  { id: 'preset', label: '预设', icon: '🏷️' },
  { id: 'non-preset', label: '非预设', icon: '📁' }
]

/** 分类选项 */
const categories = computed((): CategoryInfo[] => {
  const cats: Record<string, number> = {}
  for (const skill of presets.value) {
    cats[skill.preset.category] = (cats[skill.preset.category] || 0) + 1
  }
  return Object.entries(cats).map(([id, count]) => {
    const meta = CATEGORY_META[id as keyof typeof CATEGORY_META]
    return {
      id,
      icon: meta?.icon || '📦',
      name: meta?.name || id,
      count
    }
  })
})

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
  loading.value = true
  try {
    const data = await invoke<SkillsBySource>('get_skills_by_source_grouped')
    skillsData.value = data
  } catch (error) {
    console.error('加载技能失败:', error)
    showToast(`加载失败: ${error}`, 'error')
  } finally {
    loading.value = false
  }
}

async function loadApiKeyRequirements() {
  try {
    apiKeyRequirements.value = await invoke('get_api_key_requirements')
    apiKeyModalOpen.value = true
  } catch (error) {
    console.error('加载 API Key 要求失败:', error)
  }
}

// ============================================================================
// 折叠/展开
// ============================================================================

function toggleCollapse(group: string) {
  if (collapsedGroups.value.has(group)) {
    collapsedGroups.value.delete(group)
  } else {
    collapsedGroups.value.add(group)
  }
  collapsedGroups.value = new Set(collapsedGroups.value)
}

function isCollapsed(group: string) {
  return collapsedGroups.value.has(group)
}

// ============================================================================
// 操作处理
// ============================================================================

/** 显示安装确认对话框 */
function showInstallConfirm(skill: SkillWithStatus) {
  skillToInstall.value = skill
  installConfirmModalOpen.value = true
}

/** 关闭安装确认对话框 */
function closeInstallConfirm() {
  installConfirmModalOpen.value = false
  skillToInstall.value = null
}

/** 显示删除确认对话框 */
function showDeleteConfirm(skill: InstalledSkillInfo) {
  skillToDelete.value = skill
  deleteConfirmModalOpen.value = true
}

/** 关闭删除确认对话框 */
function closeDeleteConfirm() {
  deleteConfirmModalOpen.value = false
  skillToDelete.value = null
}

/** 显示 Toast 通知 */
function showToast(message: string, type: 'success' | 'error' | 'warning' = 'success') {
  toast.value = {
    show: true,
    message,
    type
  }

  // 3秒后自动隐藏
  setTimeout(() => {
    toast.value.show = false
  }, 3000)
}

/** 增量更新单个预设技能的状态 */
async function updatePresetSkillStatus(skillId: string) {
  try {
    const data = await invoke<SkillsBySource>('get_skills_by_source_grouped')
    skillsData.value = data
  } catch (error) {
    console.error('更新技能状态失败:', error)
  }
}

/** 执行安装操作 */
async function executeInstall(skill: SkillWithStatus) {
  closeInstallConfirm()
  installingSkillId.value = skill.preset.id

  try {
    await invoke('install_skill', { skillId: skill.preset.id })
    await updatePresetSkillStatus(skill.preset.id)
    showToast(`"${skill.preset.name}" 安装成功！`, 'success')

    // 检查是否有缺失依赖需要安装
    if (skill.missingDependencies.length > 0) {
      const guide = await invoke('get_dependency_install_guide', { skillId: skill.preset.id })
      depGuideInfo.value = guide as {
        skill_id: string
        skill_name: string
        has_missing_deps: boolean
        missing_deps: string[]
        install_commands: string[]
        has_install_config: boolean
      }
      depGuideModalOpen.value = true
    }
  } catch (error) {
    console.error('安装失败:', error)
    showToast(`"${skill.preset.name}" 安装失败：${error}`, 'error')
  } finally {
    installingSkillId.value = null
  }
}

/** 执行删除操作 */
async function executeDelete() {
  if (!skillToDelete.value) return

  const skill = skillToDelete.value
  closeDeleteConfirm()
  installingSkillId.value = skill.id

  try {
    await invoke('delete_installed_skill', { skillId: skill.id, skillPath: skill.path })
    await loadData()
    showToast(`"${skill.name}" 已删除`, 'success')
  } catch (error) {
    console.error('删除失败:', error)
    showToast(`删除失败：${error}`, 'error')
  } finally {
    installingSkillId.value = null
    skillToDelete.value = null
  }
}

/** 处理安装按钮点击 */
function handleInstall(skill: SkillWithStatus) {
  showInstallConfirm(skill)
}

/** 处理卸载按钮点击 */
async function handleUninstall(skill: SkillWithStatus) {
  installingSkillId.value = skill.preset.id
  try {
    await invoke('uninstall_skill', { skillId: skill.preset.id })
    await loadData()
    showToast(`"${skill.preset.name}" 已卸载`, 'success')
  } catch (error) {
    console.error('卸载失败:', error)
    showToast(`卸载失败：${error}`, 'error')
  } finally {
    installingSkillId.value = null
  }
}

/** 处理删除按钮点击 */
function handleDelete(skill: InstalledSkillInfo) {
  showDeleteConfirm(skill)
}

/** 处理切换启用状态 */
async function handleToggleEnabled(skill: SkillWithStatus) {
  try {
    if (skill.enabled) {
      await invoke('disable_skill', { skillId: skill.preset.id })
    } else {
      await invoke('enable_skill', { skillId: skill.preset.id })
    }
    await loadData()
  } catch (error) {
    console.error('切换启用状态失败:', error)
  }
}

/** 打开详情弹窗 - 预设技能 */
async function openPresetDetail(skill: SkillWithStatus) {
  selectedSkill.value = skill
  selectedSkillType.value = 'preset'

  // 获取文档内容
  try {
    // 尝试从 workspace 目录读取文档
    const workspacePath = `~/.openclaw/workspace/skills/${skill.preset.id}`
    const doc = await invoke<SkillDocument>('get_skill_document', {
      skillId: skill.preset.id,
      skillPath: workspacePath
    })
    skillDocument.value = doc
  } catch {
    skillDocument.value = null
  }

  detailModalOpen.value = true
}

/** 打开详情弹窗 - 已安装技能 */
async function openInstalledDetail(skill: InstalledSkillInfo) {
  selectedSkill.value = skill
  selectedSkillType.value = 'installed'

  // 获取文档内容
  try {
    const doc = await invoke<SkillDocument>('get_skill_document', {
      skillId: skill.id,
      skillPath: skill.path
    })
    skillDocument.value = doc
  } catch {
    skillDocument.value = null
  }

  detailModalOpen.value = true
}

/** 打开文件夹 */
async function openFolder(skill: InstalledSkillInfo) {
  try {
    await invoke('open_skill_folder', { skillPath: skill.path })
  } catch (error) {
    console.error('打开文件夹失败:', error)
    showToast(`打开文件夹失败：${error}`, 'error')
  }
}

function closeDetail() {
  detailModalOpen.value = false
  selectedSkill.value = null
  skillDocument.value = null
}

function closeDepGuide() {
  depGuideModalOpen.value = false
  depGuideInfo.value = null
}

// ============================================================================
// 批量操作
// ============================================================================

/** 切换技能选择状态 */
function toggleSkillSelection(skillId: string) {
  if (selectedSkillIds.value.has(skillId)) {
    selectedSkillIds.value.delete(skillId)
  } else {
    selectedSkillIds.value.add(skillId)
  }
  selectedSkillIds.value = new Set(selectedSkillIds.value)
}

/** 检查技能是否被选中 */
function isSkillSelected(skillId: string): boolean {
  return selectedSkillIds.value.has(skillId)
}

/** 全选/取消全选 */
function toggleSelectAll() {
  if (isAllSelected.value) {
    selectedSkillIds.value.clear()
  } else {
    installableSkills.value.forEach(skill => {
      selectedSkillIds.value.add(skill.preset.id)
    })
  }
  selectedSkillIds.value = new Set(selectedSkillIds.value)
}

/** 清空选择 */
function clearSelection() {
  selectedSkillIds.value.clear()
  selectedSkillIds.value = new Set()
}

/** 开始批量安装 */
async function startBatchInstall() {
  if (selectedSkillIds.value.size === 0) return

  isBatchInstalling.value = true
  batchProgress.value = { current: 0, total: selectedSkillIds.value.size }
  batchResults.value = []

  const skillIds = Array.from(selectedSkillIds.value)

  for (let i = 0; i < skillIds.length; i++) {
    const skillId = skillIds[i]
    batchProgress.value.current = i + 1

    const skill = presets.value.find(s => s.preset.id === skillId)
    if (!skill) continue

    try {
      await invoke('install_skill', { skillId: skill.preset.id })
      batchResults.value.push({
        skillId: skill.preset.id,
        skillName: skill.preset.name,
        success: true
      })
    } catch (error) {
      batchResults.value.push({
        skillId: skill.preset.id,
        skillName: skill.preset.name,
        success: false,
        message: String(error)
      })
    }
  }

  isBatchInstalling.value = false
  await loadData()
  batchModalOpen.value = true
  clearSelection()
}

/** 关闭批量结果弹窗 */
function closeBatchModal() {
  batchModalOpen.value = false
  batchResults.value = []
}

// ============================================================================
// 辅助函数
// ============================================================================

function getCategoryMeta(categoryId: string) {
  return CATEGORY_META[categoryId as keyof typeof CATEGORY_META] || {
    id: categoryId,
    name: categoryId,
    icon: '📦',
    description: ''
  }
}

function getSourceMeta(source: string) {
  return SOURCE_META[source as keyof typeof SOURCE_META] || {
    id: source,
    name: source,
    icon: '📦',
    color: 'var(--oc-text-secondary)'
  }
}

function getSkillSourceMeta(source: 'preset' | 'workspace' | 'agent') {
  return SKILL_SOURCE_META[source]
}

function isInstalling(skillId: string) {
  return installingSkillId.value === skillId
}

/** 获取 Agent 技能的显示名称 */
function getAgentDisplayName(agentName: string): string {
  // 将连字符替换为空格，首字母大写
  return agentName
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}
</script>

<template>
  <div class="skill-presets-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-title">
        <h1 style="color: var(--oc-text-primary);">📦 Skill 管理</h1>
        <p class="header-subtitle" style="color: var(--oc-text-secondary);">
          预设技能 · 已安装技能 · 非预设技能
        </p>
      </div>

      <div class="header-actions">
        <Button variant="ghost" size="sm" @click="loadApiKeyRequirements">
          <AlertTriangle class="w-4 h-4 mr-1" />
          API Key 配置
        </Button>
        <Button variant="ghost" size="sm" @click="loadData" :disabled="loading">
          <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': loading }" />
        </Button>
      </div>
    </div>

    <!-- 搜索和筛选栏 -->
    <div class="filter-bar">
      <!-- 搜索框 -->
      <div class="search-box">
        <Search class="search-icon" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索技能名称、描述或标签..."
          class="search-input"
        />
        <button v-if="searchQuery" @click="searchQuery = ''" class="search-clear">
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- 来源筛选 -->
      <div class="source-tabs">
        <button
          v-for="option in sourceOptions"
          :key="option.id"
          :class="['source-tab', { active: selectedSource === option.id }]"
          @click="selectedSource = option.id as any"
        >
          <span>{{ option.icon }}</span>
          <span>{{ option.label }}</span>
        </button>
      </div>

      <!-- 仅显示已安装 -->
      <button
        :class="['recommend-filter', { active: showInstalledOnly }]"
        @click="showInstalledOnly = !showInstalledOnly"
      >
        <Check class="w-4 h-4" />
        已安装
      </button>
    </div>

    <!-- 批量操作工具栏 -->
    <div v-if="hasSelection && selectedSource !== 'non-preset'" class="batch-toolbar">
      <div class="batch-info">
        已选择 <strong>{{ selectedCount }}</strong> 个预设技能
      </div>
      <div class="batch-actions">
        <Button
          variant="default"
          size="sm"
          @click="startBatchInstall"
          :disabled="isBatchInstalling"
        >
          <Download class="w-4 h-4 mr-1" />
          批量安装
        </Button>
        <Button
          variant="outline"
          size="sm"
          @click="clearSelection"
          :disabled="isBatchInstalling"
        >
          取消选择
        </Button>
      </div>
    </div>

    <!-- 分类标签 -->
    <div class="category-tags" v-if="selectedSource !== 'non-preset'">
      <button
        :class="['category-tag', { active: !selectedCategory }]"
        @click="selectedCategory = null"
      >
        全部
        <span class="tag-count">{{ presets.length }}</span>
      </button>
      <button
        v-for="cat in categories"
        :key="cat.id"
        :class="['category-tag', { active: selectedCategory === cat.id }]"
        @click="selectedCategory = cat.id"
      >
        <span>{{ cat.icon }}</span>
        {{ cat.name }}
        <span class="tag-count">{{ cat.count }}</span>
      </button>
    </div>

    <!-- 主内容区 -->
    <div class="main-content">
      <!-- 加载状态 -->
      <div v-if="loading" class="loading-state">
        <RefreshCw class="w-8 h-8 animate-spin" style="color: var(--oc-text-muted);" />
        <p style="color: var(--oc-text-secondary);">加载中...</p>
      </div>

      <template v-else>
        <!-- 预设技能分组 -->
        <section v-if="selectedSource !== 'non-preset'" class="section preset-section">
          <!-- 分组标题 -->
          <div class="group-header" @click="toggleCollapse('presets')">
            <div class="group-title">
              <span class="group-icon">📋</span>
              <span class="group-name">预设技能</span>
              <span class="group-count">({{ filteredPresets.length }})</span>
              <span v-if="installedPresetCount > 0" class="group-installed">
                · 已安装 {{ installedPresetCount }}
              </span>
            </div>
            <button class="collapse-btn">
              <ChevronUp v-if="!isCollapsed('presets')" class="w-4 h-4" />
              <ChevronDown v-else class="w-4 h-4" />
            </button>
          </div>

          <!-- 分组内容 -->
          <div v-if="!isCollapsed('presets')" class="group-content">
            <!-- 推荐技能子分组 -->
            <div v-if="recommendedPresets.length > 0 && !selectedCategory" class="sub-group">
              <div class="sub-group-header" @click="toggleCollapse('recommended')">
                <Star class="w-4 h-4" style="color: var(--oc-accent);" />
                <span>推荐技能</span>
                <span class="sub-group-count">({{ recommendedPresets.length }})</span>
                <button class="collapse-btn small">
                  <ChevronUp v-if="!isCollapsed('recommended')" class="w-3 h-3" />
                  <ChevronDown v-else class="w-3 h-3" />
                </button>
              </div>

              <div v-if="!isCollapsed('recommended')" class="skills-grid">
                <div
                  v-for="skill in recommendedPresets"
                  :key="skill.preset.id"
                  class="skill-card recommended"
                >
                  <div class="skill-header">
                    <div class="skill-icon">{{ skill.preset.icon }}</div>
                    <div class="skill-info">
                      <h3 class="skill-name">{{ skill.preset.name }}</h3>
                      <p class="skill-id">{{ skill.preset.id }}</p>
                    </div>
                    <Badge variant="accent" size="sm">
                      <Star class="w-3 h-3 mr-0.5" />
                      推荐
                    </Badge>
                  </div>

                  <p class="skill-description">{{ skill.preset.description }}</p>

                  <div class="skill-actions">
                    <template v-if="skill.installed">
                      <!-- 手动安装显示特殊徽章 -->
                      <Badge v-if="skill.installedFromPresetSystem === false" variant="accent" size="sm">
                        ✨ 手动安装
                      </Badge>
                      <!-- 预设系统安装显示启用/禁用状态 -->
                      <Badge v-else :variant="skill.enabled ? 'success' : 'warning'" size="sm">
                        {{ skill.enabled ? '● 已启用' : '⏸ 已禁用' }}
                      </Badge>
                      <Button
                        variant="outline"
                        size="sm"
                        @click="handleToggleEnabled(skill)"
                      >
                        {{ skill.enabled ? '禁用' : '启用' }}
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        @click="handleUninstall(skill)"
                        :disabled="isInstalling(skill.preset.id)"
                        style="color: var(--oc-danger);"
                      >
                        <Trash2 class="w-3 h-3 mr-1" />
                        卸载
                      </Button>
                    </template>
                    <template v-else>
                      <Button
                        variant="default"
                        size="sm"
                        @click="handleInstall(skill)"
                        :disabled="isInstalling(skill.preset.id)"
                      >
                        <Download class="w-3 h-3 mr-1" />
                        {{ isInstalling(skill.preset.id) ? '安装中...' : '安装' }}
                      </Button>
                    </template>
                    <Button variant="ghost" size="sm" @click="openPresetDetail(skill)">
                      详情
                      <ChevronRight class="w-3 h-3 ml-1" />
                    </Button>
                  </div>
                </div>
              </div>
            </div>

            <!-- 全部预设技能 -->
            <div class="skills-grid" v-if="filteredPresets.length > 0">
              <div
                v-for="skill in filteredPresets"
                :key="skill.preset.id"
                class="skill-card"
              >
                <div class="skill-header">
                  <div class="skill-icon">{{ skill.preset.icon }}</div>
                  <div class="skill-info">
                    <h3 class="skill-name">{{ skill.preset.name }}</h3>
                    <p class="skill-id">{{ skill.preset.id }}</p>
                  </div>
                  <Badge variant="default" size="sm">🏷️ 预设</Badge>
                </div>

                <p class="skill-description">{{ skill.preset.description }}</p>

                <div class="skill-actions">
                  <template v-if="skill.installed">
                    <!-- 手动安装显示特殊徽章 -->
                    <Badge v-if="skill.installedFromPresetSystem === false" variant="accent" size="sm">
                      ✨ 手动安装
                    </Badge>
                    <!-- 预设系统安装显示启用/禁用状态 -->
                    <Badge v-else :variant="skill.enabled ? 'success' : 'warning'" size="sm">
                      {{ skill.enabled ? '● 已启用' : '⏸ 已禁用' }}
                    </Badge>
                    <Button
                      variant="outline"
                      size="sm"
                      @click="handleToggleEnabled(skill)"
                    >
                      {{ skill.enabled ? '禁用' : '启用' }}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      @click="handleUninstall(skill)"
                      :disabled="isInstalling(skill.preset.id)"
                      style="color: var(--oc-danger);"
                    >
                      <Trash2 class="w-3 h-3 mr-1" />
                      卸载
                    </Button>
                  </template>
                  <template v-else>
                    <Button
                      variant="default"
                      size="sm"
                      @click="handleInstall(skill)"
                      :disabled="isInstalling(skill.preset.id)"
                    >
                      <Download class="w-3 h-3 mr-1" />
                      {{ isInstalling(skill.preset.id) ? '安装中...' : '安装' }}
                    </Button>
                  </template>
                  <Button variant="ghost" size="sm" @click="openPresetDetail(skill)">
                    详情
                    <ChevronRight class="w-3 h-3 ml-1" />
                  </Button>
                </div>
              </div>
            </div>

            <!-- 空状态 -->
            <div v-if="filteredPresets.length === 0" class="empty-state">
              <Search class="w-12 h-12" style="color: var(--oc-text-quiet);" />
              <p style="color: var(--oc-text-secondary);">未找到匹配的预设技能</p>
            </div>
          </div>
        </section>

        <!-- 非预设技能分组 -->
        <section v-if="selectedSource !== 'preset'" class="section non-preset-section">
          <!-- 分组标题 -->
          <div class="group-header" @click="toggleCollapse('non-preset')">
            <div class="group-title">
              <span class="group-icon">📁</span>
              <span class="group-name">非预设技能</span>
              <span class="group-count">({{ nonPresetCount }})</span>
            </div>
            <button class="collapse-btn">
              <ChevronUp v-if="!isCollapsed('non-preset')" class="w-4 h-4" />
              <ChevronDown v-else class="w-4 h-4" />
            </button>
          </div>

          <!-- 分组内容 -->
          <div v-if="!isCollapsed('non-preset')" class="group-content">
            <!-- 提示信息 -->
            <div class="non-preset-tip">
              <AlertTriangle class="w-4 h-4" />
              <span>这些技能已直接安装，非来自预设</span>
            </div>

            <!-- 主Agent技能 - 纯非预设 -->
            <div v-if="filteredPureNonPresetWorkspace.length > 0" class="sub-group">
              <div class="sub-group-header">
                <span class="sub-group-icon">🏠</span>
                <span>主Agent</span>
                <span class="sub-group-count">({{ filteredPureNonPresetWorkspace.length }})</span>
              </div>

              <div class="skills-grid">
                <div
                  v-for="skill in filteredPureNonPresetWorkspace"
                  :key="skill.id"
                  class="skill-card non-preset"
                >
                  <div class="skill-header">
                    <div class="skill-icon">🔧</div>
                    <div class="skill-info">
                      <h3 class="skill-name">{{ skill.name || skill.id }}</h3>
                      <p class="skill-id">{{ skill.id }}</p>
                    </div>
                    <Badge variant="success" size="sm">🏠 主Agent</Badge>
                  </div>

                  <p class="skill-description">{{ skill.description || '暂无描述' }}</p>

                  <div class="skill-actions">
                    <Button
                      variant="ghost"
                      size="sm"
                      @click="handleDelete(skill)"
                      :disabled="isInstalling(skill.id)"
                      style="color: var(--oc-danger);"
                    >
                      <Trash2 class="w-3 h-3 mr-1" />
                      删除
                    </Button>
                    <Button variant="ghost" size="sm" @click="openInstalledDetail(skill)">
                      详情
                      <ChevronRight class="w-3 h-3 ml-1" />
                    </Button>
                  </div>
                </div>
              </div>
            </div>

            <!-- Agent技能 - 纯非预设 -->
            <div v-for="(skills, agentName) in filteredPureNonPresetAgent" :key="agentName" class="sub-group">
              <div class="sub-group-header">
                <span class="sub-group-icon">🤖</span>
                <span>{{ getAgentDisplayName(agentName) }}</span>
                <span class="sub-group-count">({{ skills.length }})</span>
              </div>

              <div class="skills-grid">
                <div
                  v-for="skill in skills"
                  :key="skill.id"
                  class="skill-card non-preset"
                >
                  <div class="skill-header">
                    <div class="skill-icon">🔧</div>
                    <div class="skill-info">
                      <h3 class="skill-name">{{ skill.name || skill.id }}</h3>
                      <p class="skill-id">{{ skill.id }}</p>
                    </div>
                    <Badge variant="default" size="sm">🤖 {{ getAgentDisplayName(agentName) }}</Badge>
                  </div>

                  <p class="skill-description">{{ skill.description || '暂无描述' }}</p>

                  <div class="skill-actions">
                    <Button
                      variant="ghost"
                      size="sm"
                      @click="handleDelete(skill)"
                      :disabled="isInstalling(skill.id)"
                      style="color: var(--oc-danger);"
                    >
                      <Trash2 class="w-3 h-3 mr-1" />
                      删除
                    </Button>
                    <Button variant="ghost" size="sm" @click="openInstalledDetail(skill)">
                      详情
                      <ChevronRight class="w-3 h-3 ml-1" />
                    </Button>
                  </div>
                </div>
              </div>
            </div>

            <!-- 空状态 -->
            <div v-if="nonPresetCount === 0" class="empty-state">
              <FolderOpen class="w-12 h-12" style="color: var(--oc-text-quiet);" />
              <p style="color: var(--oc-text-secondary);">暂无非预设技能</p>
            </div>
          </div>
        </section>
      </template>
    </div>

    <!-- 详情弹窗 -->
    <Teleport to="body">
      <div v-if="detailModalOpen && selectedSkill" class="modal-overlay" @click.self="closeDetail">
        <div class="modal-content skill-detail-modal">
          <div class="modal-header">
            <div class="modal-title">
              <div class="modal-icon">
                {{ selectedSkillType === 'preset' ? (selectedSkill as SkillWithStatus).preset.icon : '🔧' }}
              </div>
              <div>
                <h2>{{ selectedSkillType === 'preset' ? (selectedSkill as SkillWithStatus).preset.name : (selectedSkill as InstalledSkillInfo).name || (selectedSkill as InstalledSkillInfo).id }}</h2>
                <p class="modal-sub-id">{{ (selectedSkill as any).preset?.id || (selectedSkill as InstalledSkillInfo).id }}</p>
              </div>
            </div>
            <Button variant="ghost" size="sm" @click="closeDetail">
              <X class="w-5 h-5" />
            </Button>
          </div>

          <div class="modal-body">
            <!-- 来源和状态 -->
            <div class="detail-meta">
              <Badge v-if="selectedSkillType === 'preset'" variant="default" size="sm">🏷️ 预设</Badge>
              <Badge v-else-if="(selectedSkill as InstalledSkillInfo).source === 'workspace'" variant="success" size="sm">🏠 主Agent</Badge>
              <Badge v-else variant="default" size="sm">🤖 {{ getAgentDisplayName((selectedSkill as InstalledSkillInfo).agentName || '') }}</Badge>

              <template v-if="selectedSkillType === 'preset'">
                <span v-if="(selectedSkill as SkillWithStatus).installed" class="status-text">
                  · {{ (selectedSkill as SkillWithStatus).enabled ? '● 已启用' : '⏸ 已禁用' }}
                </span>
                <span v-else class="status-text">· 未安装</span>
              </template>
            </div>

            <!-- 文档内容 -->
            <div v-if="skillDocument?.content" class="detail-document">
              <h3>📄 说明文档</h3>
              <div class="document-content">
                <pre>{{ skillDocument.content }}</pre>
              </div>
            </div>
            <div v-else class="detail-no-document">
              <p>暂无说明文档</p>
            </div>

            <!-- 路径 -->
            <div v-if="selectedSkillType === 'installed'" class="detail-path">
              <h3>📍 路径</h3>
              <code>{{ (selectedSkill as InstalledSkillInfo).path }}</code>
            </div>
          </div>

          <div class="modal-footer">
            <template v-if="selectedSkillType === 'preset'">
              <Button variant="outline" @click="closeDetail">关闭</Button>
              <template v-if="(selectedSkill as SkillWithStatus).installed">
                <Button
                  variant="outline"
                  @click="handleToggleEnabled(selectedSkill as SkillWithStatus); closeDetail()"
                >
                  {{ (selectedSkill as SkillWithStatus).enabled ? '禁用' : '启用' }}
                </Button>
                <Button
                  variant="destructive"
                  @click="handleUninstall(selectedSkill as SkillWithStatus); closeDetail()"
                >
                  <Trash2 class="w-4 h-4 mr-1" />
                  卸载
                </Button>
              </template>
              <template v-else>
                <Button
                  variant="default"
                  @click="handleInstall(selectedSkill as SkillWithStatus); closeDetail()"
                >
                  <Download class="w-4 h-4 mr-1" />
                  安装
                </Button>
              </template>
            </template>
            <template v-else>
              <Button variant="outline" @click="openFolder(selectedSkill as InstalledSkillInfo)">
                <FolderOpen class="w-4 h-4 mr-1" />
                定位
              </Button>
              <Button variant="destructive" @click="handleDelete(selectedSkill as InstalledSkillInfo); closeDetail()">
                <Trash2 class="w-4 h-4 mr-1" />
                删除
              </Button>
            </template>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- 删除确认弹窗 -->
    <Teleport to="body">
      <div v-if="deleteConfirmModalOpen && skillToDelete" class="modal-overlay" @click.self="closeDeleteConfirm">
        <div class="modal-content delete-confirm-modal">
          <div class="modal-header">
            <h2>
              <AlertTriangle class="w-5 h-5 mr-2" style="color: var(--oc-danger);" />
              确认删除非预设技能
            </h2>
            <Button variant="ghost" size="sm" @click="closeDeleteConfirm">
              <X class="w-5 h-5" />
            </Button>
          </div>

          <div class="modal-body">
            <p class="delete-skill-name">您确定要删除 "<strong>{{ skillToDelete.name || skillToDelete.id }}</strong>" 吗？</p>

            <div class="delete-warning">
              <AlertTriangle class="w-4 h-4" />
              <div>
                <strong>警告：</strong>
                <ul>
                  <li>此技能来自非预设来源</li>
                  <li>将永久删除此技能及其所有文件</li>
                  <li>此操作不可撤销</li>
                </ul>
              </div>
            </div>
          </div>

          <div class="modal-footer">
            <Button variant="outline" @click="closeDeleteConfirm">取消</Button>
            <Button variant="destructive" @click="executeDelete">
              <Trash2 class="w-4 h-4 mr-1" />
              确认删除
            </Button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Toast 通知 -->
    <Teleport to="body">
      <Transition name="toast">
        <div v-if="toast.show" :class="['toast-notification', `toast-${toast.type}`]">
          <Check v-if="toast.type === 'success'" class="w-5 h-5" />
          <AlertTriangle v-else-if="toast.type === 'warning'" class="w-5 h-5" />
          <X v-else class="w-5 h-5" />
          <span>{{ toast.message }}</span>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.skill-presets-page {
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
  gap: 0.5rem;
  align-items: center;
}

/* 筛选栏 */
.filter-bar {
  display: flex;
  gap: 1rem;
  padding: 1rem 1.5rem;
  align-items: center;
  flex-wrap: wrap;
  border-bottom: 1px solid var(--oc-divider);
}

.search-box {
  position: relative;
  flex: 1;
  min-width: 200px;
  max-width: 400px;
}

.search-icon {
  position: absolute;
  left: 0.75rem;
  top: 50%;
  transform: translateY(-50%);
  color: var(--oc-text-muted);
  width: 1rem;
  height: 1rem;
}

.search-input {
  width: 100%;
  padding: 0.5rem 2.5rem 0.5rem 2.25rem;
  border: 1px solid var(--oc-divider);
  border-radius: 0.5rem;
  background: var(--oc-input);
  color: var(--oc-text-primary);
  font-size: 0.875rem;
}

.search-input:focus {
  outline: none;
  border-color: var(--oc-input-focus);
}

.search-clear {
  position: absolute;
  right: 0.5rem;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  padding: 0.25rem;
  cursor: pointer;
  color: var(--oc-text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 0.25rem;
}

.search-clear:hover {
  background: var(--oc-item-hover);
  color: var(--oc-text-primary);
}

/* 来源标签 */
.source-tabs {
  display: flex;
  background: var(--oc-card);
  border-radius: 0.5rem;
  padding: 0.25rem;
  border: 1px solid var(--oc-divider);
}

.source-tab {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  border: none;
  background: none;
  color: var(--oc-text-secondary);
  font-size: 0.875rem;
  cursor: pointer;
  border-radius: 0.375rem;
  transition: all 0.15s ease;
}

.source-tab:hover {
  color: var(--oc-text-primary);
}

.source-tab.active {
  background: var(--oc-accent);
  color: white;
}

/* 推荐筛选 */
.recommend-filter {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  border: 1px solid var(--oc-divider);
  border-radius: 0.5rem;
  background: var(--oc-card);
  color: var(--oc-text-secondary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.recommend-filter:hover {
  border-color: var(--oc-accent);
  color: var(--oc-text-primary);
}

.recommend-filter.active {
  background: color-mix(in srgb, var(--oc-accent) 15%, transparent);
  border-color: var(--oc-accent);
  color: var(--oc-accent);
}

/* 分类标签 */
.category-tags {
  display: flex;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  overflow-x: auto;
  border-bottom: 1px solid var(--oc-divider);
}

.category-tag {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  border: 1px solid var(--oc-divider);
  border-radius: 9999px;
  background: var(--oc-card);
  color: var(--oc-text-secondary);
  font-size: 0.8125rem;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s ease;
}

.category-tag:hover {
  border-color: var(--oc-text-secondary);
  color: var(--oc-text-primary);
}

.category-tag.active {
  background: var(--oc-accent);
  border-color: var(--oc-accent);
  color: white;
}

.tag-count {
  padding: 0 0.375rem;
  background: color-mix(in srgb, var(--oc-text-muted) 20%, transparent);
  border-radius: 9999px;
  font-size: 0.75rem;
}

.category-tag.active .tag-count {
  background: rgba(255, 255, 255, 0.25);
}

/* 主内容区 */
.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

/* 加载状态 */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 3rem;
}

/* 分组 */
.section {
  margin-bottom: 2rem;
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.25rem;
  background: var(--oc-card);
  border: 1px solid var(--oc-divider);
  border-radius: 0.75rem;
  margin-bottom: 1rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.group-header:hover {
  border-color: var(--oc-divider-soft);
}

.group-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.group-icon {
  font-size: 1.25rem;
}

.group-name {
  font-size: 1rem;
  font-weight: 600;
  color: var(--oc-text-primary);
}

.group-count {
  font-size: 0.875rem;
  color: var(--oc-text-muted);
}

.group-installed {
  font-size: 0.875rem;
  color: var(--oc-success);
}

.collapse-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border: none;
  background: none;
  color: var(--oc-text-muted);
  cursor: pointer;
  border-radius: 0.25rem;
  transition: all 0.15s ease;
}

.collapse-btn:hover {
  background: var(--oc-item-hover);
  color: var(--oc-text-primary);
}

.collapse-btn.small {
  width: 1.25rem;
  height: 1.25rem;
}

.group-content {
  padding-left: 0.5rem;
}

/* 子分组 */
.sub-group {
  margin-bottom: 1.5rem;
}

.sub-group-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  margin-bottom: 0.75rem;
  color: var(--oc-text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  border-radius: 0.375rem;
  transition: all 0.15s ease;
}

.sub-group-header:hover {
  background: var(--oc-item-hover);
  color: var(--oc-text-primary);
}

.sub-group-icon {
  font-size: 1rem;
}

.sub-group-count {
  color: var(--oc-text-muted);
}

/* 技能网格 */
.skills-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1rem;
}

/* 技能卡片 */
.skill-card {
  background: var(--oc-card);
  border: 1px solid var(--oc-divider);
  border-radius: 0.75rem;
  padding: 1rem;
  transition: all 0.15s ease;
}

.skill-card:hover {
  border-color: var(--oc-divider-soft);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.skill-card.recommended {
  border-color: color-mix(in srgb, var(--oc-accent) 30%, transparent);
  background: color-mix(in srgb, var(--oc-accent) 5%, var(--oc-card));
}

.skill-card.non-preset {
  border-left: 3px solid var(--oc-success);
}

.skill-header {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.skill-icon {
  font-size: 2rem;
  line-height: 1;
  flex-shrink: 0;
}

.skill-info {
  flex: 1;
  min-width: 0;
}

.skill-name {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--oc-text-primary);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.skill-id {
  font-size: 0.75rem;
  font-weight: 400;
  color: var(--oc-text-secondary);
  margin: 0.125rem 0 0 0;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.8;
}

.skill-description {
  font-size: 0.8125rem;
  color: var(--oc-text-secondary);
  margin: 0 0 0.75rem;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.skill-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  padding-top: 0.75rem;
  border-top: 1px solid var(--oc-divider);
  flex-wrap: wrap;
}

.skill-actions :deep(.oc-button) {
  font-size: 0.8125rem;
}

/* 非预设提示 */
.non-preset-tip {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: color-mix(in srgb, var(--oc-warning) 10%, transparent);
  border-radius: 0.5rem;
  margin-bottom: 1rem;
  font-size: 0.875rem;
  color: var(--oc-warning);
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 3rem;
  text-align: center;
}

/* 批量操作栏 */
.batch-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1.5rem;
  background: color-mix(in srgb, var(--oc-accent) 5%, transparent);
  border-bottom: 1px solid var(--oc-accent);
}

.batch-info {
  font-size: 0.875rem;
  color: var(--oc-text-secondary);
}

.batch-actions {
  display: flex;
  gap: 0.5rem;
}

/* 弹窗样式 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1rem;
}

.modal-content {
  background: var(--oc-card);
  border-radius: 1rem;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.skill-detail-modal {
  width: 100%;
  max-width: 600px;
}

.delete-confirm-modal {
  width: 100%;
  max-width: 480px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--oc-divider);
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.modal-icon {
  font-size: 2.5rem;
  line-height: 1;
}

.modal-title h2 {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--oc-text-primary);
  margin: 0;
}

.modal-sub-id {
  font-size: 0.875rem;
  font-weight: 400;
  color: var(--oc-text-secondary);
  margin: 0.25rem 0 0;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--oc-divider);
}

/* 详情弹窗内容 */
.detail-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1.5rem;
}

.status-text {
  font-size: 0.875rem;
  color: var(--oc-text-secondary);
}

.detail-document {
  margin-bottom: 1.5rem;
}

.detail-document h3 {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--oc-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0 0 0.75rem;
}

.document-content {
  background: var(--oc-bg-secondary);
  border: 1px solid var(--oc-divider);
  border-radius: 0.5rem;
  padding: 1rem;
  max-height: 300px;
  overflow-y: auto;
}

.document-content pre {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--oc-text-primary);
  white-space: pre-wrap;
  word-break: break-word;
  font-family: inherit;
}

.detail-no-document {
  padding: 2rem;
  text-align: center;
  color: var(--oc-text-muted);
}

.detail-path {
  margin-bottom: 1rem;
}

.detail-path h3 {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--oc-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0 0 0.5rem;
}

.detail-path code {
  display: block;
  padding: 0.5rem;
  background: var(--oc-card-elevated);
  border-radius: 0.25rem;
  font-size: 0.75rem;
  color: var(--oc-text-secondary);
  word-break: break-all;
}

/* 删除确认弹窗 */
.delete-skill-name {
  font-size: 0.9375rem;
  color: var(--oc-text-primary);
  margin: 0 0 1rem;
}

.delete-warning {
  display: flex;
  gap: 0.75rem;
  padding: 1rem;
  background: color-mix(in srgb, var(--oc-danger) 10%, transparent);
  border-radius: 0.5rem;
  color: var(--oc-danger);
}

.delete-warning ul {
  margin: 0.5rem 0 0 1.25rem;
  font-size: 0.875rem;
}

.delete-warning li {
  margin-bottom: 0.25rem;
}

/* Toast 通知 */
.toast-notification {
  position: fixed;
  bottom: 2rem;
  right: 2rem;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem 1.25rem;
  border-radius: 0.75rem;
  font-size: 0.875rem;
  font-weight: 500;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
  z-index: 2000;
  min-width: 320px;
  max-width: 480px;
}

.toast-success {
  background: var(--oc-success);
  color: white;
}

.toast-error {
  background: var(--oc-error);
  color: white;
}

.toast-warning {
  background: var(--oc-warning);
  color: var(--oc-text-primary);
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.toast-enter-from {
  opacity: 0;
  transform: translateY(20px) translateX(20px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateY(20px) translateX(20px);
}

/* 动画 */
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.animate-spin {
  animation: spin 1s linear infinite;
}

/* 响应式 */
@media (max-width: 768px) {
  .toast-notification {
    right: 1rem;
    left: 1rem;
    min-width: 0;
  }

  .skills-grid {
    grid-template-columns: 1fr;
  }
}
</style>
