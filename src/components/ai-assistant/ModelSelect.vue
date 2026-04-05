<template>
  <div class="model-select" ref="containerRef">
    <!-- 触发按钮 -->
    <button
      type="button"
      class="select-trigger"
      :class="{ open: isOpen, disabled }"
      :disabled="disabled"
      @click="toggle"
      @keydown.esc="close"
      @keydown.enter.prevent="toggle"
    >
      <span class="selected-label">
        {{ selectedModelInfo ? selectedModelInfo.display : placeholder }}
      </span>
      <ChevronDown :size="16" class="chevron" :class="{ rotated: isOpen }" />
    </button>

    <!-- 下拉面板 -->
    <Teleport to="body">
      <div
        v-if="isOpen"
        class="select-dropdown"
        :style="dropdownStyle"
        @click.stop
      >
        <!-- 搜索框 -->
        <div class="search-wrapper">
          <Search :size="14" class="search-icon" />
          <input
            ref="searchInput"
            v-model="searchQuery"
            type="text"
            class="search-input"
            placeholder="搜索模型..."
            @keydown.esc="close"
            @keydown.down.prevent="moveFocus(1)"
            @keydown.up.prevent="moveFocus(-1)"
            @keydown.enter.prevent="selectFocused"
          />
        </div>

        <!-- 模型列表 -->
        <div class="model-list" ref="listRef">
          <template v-if="groupedModels.length > 0">
            <div
              v-for="group in groupedModels"
              :key="group.series"
              class="model-group"
            >
              <div class="group-label">{{ group.series }}</div>
              <button
                v-for="(model, idx) in group.models"
                :key="model"
                type="button"
                :class="['model-option', { selected: model === modelValue, focused: focusedIndex === getGlobalIndex(group.series, idx) }]"
                @click="select(model)"
                @mouseenter="focusedIndex = getGlobalIndex(group.series, idx)"
              >
                <span class="model-name">{{ model }}</span>
                <Check v-if="model === modelValue" :size="14" class="check-icon" />
              </button>
            </div>
          </template>
          <div v-else class="no-results">
            <span>未找到匹配模型</span>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { ChevronDown, Search, Check } from 'lucide-vue-next'

interface Props {
  modelValue: string
  models: string[]
  placeholder?: string
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '选择模型...',
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

// --- State ---
const isOpen = ref(false)
const searchQuery = ref('')
const focusedIndex = ref(0)
const containerRef = ref<HTMLElement>()
const searchInput = ref<HTMLInputElement>()
const listRef = ref<HTMLElement>()
const dropdownStyle = ref<Record<string, string>>({ top: '0px', left: '0px', width: '0px' })

// --- Computed: group models by series ---
const seriesOrder = ['Opus', 'Sonnet', 'Haiku', 'GPT-4', 'GPT-3.5', 'O1', 'O3', 'O4', 'Mini', 'Turbo', 'Default']

function detectSeries(model: string): string {
  const m = model.toLowerCase()
  if (m.includes('opus')) return 'Opus'
  if (m.includes('sonnet')) return 'Sonnet'
  if (m.includes('haiku')) return 'Haiku'
  if (m.includes('o1') || m.includes('o2') || m.includes('o3') || m.includes('o4')) return 'O-Series'
  if (m.includes('gpt-4')) return 'GPT-4'
  if (m.includes('gpt-3.5')) return 'GPT-3.5'
  if (m.includes('mini')) return 'Mini'
  if (m.includes('turbo')) return 'Turbo'
  return 'Default'
}

const groupedModels = computed(() => {
  const query = searchQuery.value.toLowerCase()
  const filtered = props.models.filter(m =>
    m.toLowerCase().includes(query)
  )

  const groups: Record<string, string[]> = {}
  for (const model of filtered) {
    const series = detectSeries(model)
    if (!groups[series]) groups[series] = []
    groups[series].push(model)
  }

  // Sort series by preferred order
  const sortedSeries = Object.keys(groups).sort((a, b) => {
    const aIdx = seriesOrder.findIndex(s => a.includes(s))
    const bIdx = seriesOrder.findIndex(s => b.includes(s))
    const aScore = aIdx === -1 ? 999 : aIdx
    const bScore = bIdx === -1 ? 999 : bIdx
    return aScore - bScore
  })

  return sortedSeries.map(series => ({
    series,
    models: groups[series]
  }))
})

const allFlat = computed(() =>
  groupedModels.value.flatMap(g => g.models)
)

const selectedModelInfo = computed(() => {
  if (!props.modelValue) return null
  return { display: props.modelValue }
})

// --- Methods ---
function getGlobalIndex(series: string, localIdx: number): number {
  let offset = 0
  for (const g of groupedModels.value) {
    if (g.series === series) return offset + localIdx
    offset += g.models.length
  }
  return localIdx
}

function toggle() {
  if (props.disabled) return
  if (isOpen.value) {
    close()
  } else {
    open()
  }
}

function open() {
  isOpen.value = true
  focusedIndex.value = props.modelValue
    ? allFlat.value.indexOf(props.modelValue)
    : 0
  nextTick(() => {
    positionDropdown()
    searchInput.value?.focus()
  })
}

function close() {
  isOpen.value = false
  searchQuery.value = ''
}

function select(model: string) {
  emit('update:modelValue', model)
  close()
}

function selectFocused() {
  const model = allFlat.value[focusedIndex.value]
  if (model) select(model)
}

function moveFocus(dir: number) {
  const len = allFlat.value.length
  if (len === 0) return
  focusedIndex.value = (focusedIndex.value + dir + len) % len
  // Scroll into view
  nextTick(() => {
    const el = listRef.value?.querySelector('.model-option.focused') as HTMLElement
    el?.scrollIntoView({ block: 'nearest' })
  })
}

function positionDropdown() {
  if (!containerRef.value) return
  const rect = containerRef.value.getBoundingClientRect()
  const dropdownHeight = 320
  const viewportHeight = window.innerHeight
  const spaceBelow = viewportHeight - rect.bottom - 4
  const spaceAbove = rect.top - 4

  // 垂直方向：如果下方空间不足且上方空间更大，则向上展开
  const flipVertical = spaceBelow < dropdownHeight && spaceAbove > spaceBelow
  const top = flipVertical
    ? `${rect.top - dropdownHeight - 4}px`
    : `${rect.bottom + 4}px`

  // 水平方向：检测右侧溢出
  const dropdownWidth = rect.width
  const viewportWidth = window.innerWidth
  let left = `${rect.left}px`
  if (rect.left + dropdownWidth > viewportWidth - 8) {
    left = `${Math.max(8, viewportWidth - dropdownWidth - 8)}px`
  }

  dropdownStyle.value = {
    top,
    left,
    width: `${rect.width}px`,
  }
}

// --- Click outside ---
function handleOutsideClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!containerRef.value?.contains(target) && !target.closest('.select-dropdown')) {
    close()
  }
}

function handleResize() {
  if (isOpen.value) {
    positionDropdown()
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleOutsideClick)
  window.addEventListener('resize', handleResize)
  window.addEventListener('scroll', handleResize, true)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleOutsideClick)
  window.removeEventListener('resize', handleResize)
  window.removeEventListener('scroll', handleResize, true)
})

// Reset focus when search changes
watch(searchQuery, () => { focusedIndex.value = 0 })
</script>

<style scoped>
.model-select {
  position: relative;
  width: 100%;
}

/* Trigger */
.select-trigger {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg-surface-elevated);
  border: 1px solid var(--oc-card-border);
  border-radius: var(--radius-lg);
  color: var(--oc-text-primary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
  text-align: left;
}

.select-trigger:hover:not(.disabled) {
  border-color: var(--oc-card-border-strong);
}

.select-trigger.open {
  border-color: var(--oc-accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--oc-accent) 15%, transparent);
}

.select-trigger.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.selected-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--oc-text-primary);
}

.selected-label:has(+ .placeholder) {
  color: var(--oc-text-placeholder);
}

.chevron {
  color: var(--oc-text-tertiary);
  flex-shrink: 0;
  transition: transform 0.25s ease;
}

.chevron.rotated {
  transform: rotate(-180deg);
}

/* Dropdown */
.select-dropdown {
  position: fixed;
  z-index: 9999;
  background: var(--bg-surface-elevated);
  border: 1px solid var(--oc-card-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  animation: dropdownIn 0.15s ease-out;
  max-height: 320px;
  display: flex;
  flex-direction: column;
}

@keyframes dropdownIn {
  from {
    opacity: 0;
    transform: translateY(-4px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* Search */
.search-wrapper {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 0.875rem;
  border-bottom: 1px solid var(--oc-divider);
  flex-shrink: 0;
}

.search-icon {
  color: var(--oc-text-tertiary);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  background: transparent;
  outline: none;
  font-size: 0.8125rem;
  color: var(--oc-text-primary);
}

.search-input::placeholder {
  color: var(--oc-text-placeholder);
}

/* Model List */
.model-list {
  overflow-y: auto;
  flex: 1;
  padding: 0.375rem;
}

.model-list::-webkit-scrollbar {
  width: 4px;
}
.model-list::-webkit-scrollbar-track { background: transparent; }
.model-list::-webkit-scrollbar-thumb {
  background: var(--oc-divider-soft);
  border-radius: 2px;
}

.model-group {
  margin-bottom: 0.25rem;
}

.group-label {
  padding: 0.375rem 0.625rem 0.25rem;
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--oc-text-tertiary);
}

.model-option {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 0.5rem 0.625rem;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  font-size: 0.8125rem;
  color: var(--oc-text-primary);
  cursor: pointer;
  transition: background 0.15s ease;
  text-align: left;
}

.model-option:hover,
.model-option.focused {
  background: var(--oc-item-hover);
}

.model-option.selected {
  color: var(--oc-accent);
  font-weight: 500;
}

.model-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.check-icon {
  color: var(--oc-accent);
  flex-shrink: 0;
}

.no-results {
  padding: 1.5rem;
  text-align: center;
  color: var(--oc-text-tertiary);
  font-size: 0.8125rem;
}
</style>
