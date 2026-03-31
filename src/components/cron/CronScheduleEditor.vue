<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Clock, Calendar, AlertTriangle, Code } from 'lucide-vue-next'
import CompactSelect from '@/components/ui/CompactSelect.vue'
import type { CompactOption } from '@/components/ui/CompactSelect.vue'
import { buildCronExpr, parseScheduleConfig, WEEKDAY_OPTIONS } from '@/utils/cronBuilder'
import { describeSchedule } from '@/utils/cronParser'
import { isValidScheduleExpr } from '@/utils/cronValidator'
import { QUICK_PRESETS } from '@/config/cronPresets'
import type { ScheduleFreq } from '@/types/cron'

// ============================================================================
// Props & Emits
// ============================================================================

interface Props {
  modelValue: string
  timezone?: string
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  timezone: 'Asia/Shanghai',
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

// ============================================================================
// 频率 Tab 配置
// ============================================================================

const FREQ_TABS: Array<{ value: ScheduleFreq; label: string }> = [
  { value: 'hourly', label: '每小时' },
  { value: 'daily', label: '每天' },
  { value: 'weekly', label: '每周' },
  { value: 'monthly', label: '每月' },
  { value: 'custom', label: '自定义' },
]

// ============================================================================
// 内部状态
// ============================================================================

/** 当前选中的频率 Tab */
const activeFreq = ref<ScheduleFreq>('daily')

/** 每小时配置 */
const hourlyMinute = ref(0)

/** 每天配置 */
const dailyHour = ref(9)
const dailyMinute = ref(0)

/** 每周配置 */
const weeklyDays = ref<number[]>([1, 2, 3, 4, 5])
const weeklyHour = ref(9)
const weeklyMinute = ref(0)

/** 每月配置 */
const monthlyDay = ref<number | 'L'>(1)
const monthlyHour = ref(0)
const monthlyMinute = ref(0)

/** 自定义表达式 */
const customExpr = ref('')

/** 是否为非标准格式 */
const isNonStandard = ref(false)

/** 防循环标志 */
let isUpdatingFromProp = false

// ============================================================================
// 表达式生成
// ============================================================================

/** 根据当前内部配置生成表达式 */
function buildCurrentExpr(): string {
  switch (activeFreq.value) {
    case 'hourly':
      return buildCronExpr({ freq: 'hourly', config: { minute: hourlyMinute.value } })
    case 'daily':
      return buildCronExpr({ freq: 'daily', config: { hour: dailyHour.value, minute: dailyMinute.value } })
    case 'weekly':
      return buildCronExpr({ freq: 'weekly', config: { days: weeklyDays.value, hour: weeklyHour.value, minute: weeklyMinute.value } })
    case 'monthly':
      return buildCronExpr({ freq: 'monthly', config: { day: monthlyDay.value, hour: monthlyHour.value, minute: monthlyMinute.value } })
    case 'custom':
      return customExpr.value
  }
}

/** 当前生成的表达式 */
const generatedExpr = computed(() => buildCurrentExpr())

/** 当前配置的人类可读描述 */
const scheduleDescription = computed(() => {
  return describeSchedule(generatedExpr.value, props.timezone)
})

/** 当前表达式是否有效 */
const exprValid = computed(() => {
  const expr = generatedExpr.value.trim()
  if (!expr) return null
  return isValidScheduleExpr(expr)
})

// ============================================================================
// 从外部 prop 同步到内部状态
// ============================================================================

function syncFromProp(expr: string) {
  if (!expr || !expr.trim()) {
    // 空表达式，保持默认（每天 09:00）
    activeFreq.value = 'daily'
    dailyHour.value = 9
    dailyMinute.value = 0
    customExpr.value = ''
    isNonStandard.value = false
    return
  }

  const parsed = parseScheduleConfig(expr)
  isNonStandard.value = parsed.isNonStandard

  switch (parsed.freq) {
    case 'hourly':
      activeFreq.value = 'hourly'
      hourlyMinute.value = parsed.config.config.minute
      break
    case 'daily':
      activeFreq.value = 'daily'
      dailyHour.value = parsed.config.config.hour
      dailyMinute.value = parsed.config.config.minute
      break
    case 'weekly':
      activeFreq.value = 'weekly'
      weeklyDays.value = parsed.config.config.days
      weeklyHour.value = parsed.config.config.hour
      weeklyMinute.value = parsed.config.config.minute
      break
    case 'monthly':
      activeFreq.value = 'monthly'
      monthlyDay.value = parsed.config.config.day
      monthlyHour.value = parsed.config.config.hour
      monthlyMinute.value = parsed.config.config.minute
      break
    case 'custom':
      activeFreq.value = 'custom'
      customExpr.value = parsed.config.expr
      break
  }
}

// ============================================================================
// Watchers
// ============================================================================

// 外部值变化 → 同步到内部
watch(
  () => props.modelValue,
  (newVal) => {
    if (isUpdatingFromProp) return
    syncFromProp(newVal)
  },
  { immediate: true }
)

// 内部配置变化 → emit 表达式给外部
watch(
  [activeFreq, hourlyMinute, dailyHour, dailyMinute, weeklyDays, weeklyHour, weeklyMinute, monthlyDay, monthlyHour, monthlyMinute, customExpr],
  () => {
    if (isUpdatingFromProp) return
    isUpdatingFromProp = true
    emit('update:modelValue', generatedExpr.value)
    // 使用 nextTick 确保 Vue 更新周期完成后再解锁
    setTimeout(() => { isUpdatingFromProp = false }, 0)
  },
  { deep: true }
)

// ============================================================================
// 交互方法
// ============================================================================

function switchFreq(freq: ScheduleFreq) {
  activeFreq.value = freq
}

function toggleWeekday(day: number) {
  const idx = weeklyDays.value.indexOf(day)
  if (idx >= 0) {
    weeklyDays.value.splice(idx, 1)
  } else {
    weeklyDays.value.push(day)
  }
}

function applyPreset(expr: string) {
  customExpr.value = expr
}

// ============================================================================
// 辅助选项（CompactSelect 格式）
// ============================================================================

/** 分钟选项（0-59，步长 5） */
const minuteOptions: CompactOption[] = Array.from({ length: 12 }, (_, i) => ({
  value: i * 5,
  label: `${pad(i * 5)}`,
}))

/** 小时选项（0-23） */
const hourOptions: CompactOption[] = Array.from({ length: 24 }, (_, i) => ({
  value: i,
  label: `${pad(i)}`,
}))

/** 每小时分钟选项（带 "XX 分" 后缀） */
const hourlyMinuteOptions: CompactOption[] = Array.from({ length: 12 }, (_, i) => ({
  value: i * 5,
  label: `${pad(i * 5)} 分`,
}))

/** 月份日期选项（1-31 + L） */
const dayOptions: CompactOption[] = [
  ...Array.from({ length: 31 }, (_, i) => ({
    value: i + 1,
    label: `${i + 1} 日`,
  })),
  { value: 'L', label: '最后一天' },
]

function pad(n: number): string {
  return n.toString().padStart(2, '0')
}
</script>

<template>
  <div class="cron-schedule-editor" :class="{ disabled }">
    <!-- 频率 Tab -->
    <div class="freq-tabs">
      <button
        v-for="tab in FREQ_TABS"
        :key="tab.value"
        class="freq-tab"
        :class="{ active: activeFreq === tab.value }"
        :disabled="disabled"
        @click="switchFreq(tab.value)"
      >
        {{ tab.label }}
      </button>
    </div>

    <!-- 分割线 -->
    <div class="tab-divider"></div>

    <!-- 每小时 Tab -->
    <div v-if="activeFreq === 'hourly'" class="freq-content">
      <div class="config-row">
        <span class="config-label">每小时的第</span>
        <CompactSelect
          :model-value="hourlyMinute"
          :options="hourlyMinuteOptions"
          :disabled="disabled"
          min-width="72px"
          @update:model-value="v => hourlyMinute = v as number"
        />
        <span class="config-label">执行</span>
      </div>
      <div class="expr-preview">
        <Code class="expr-icon" :size="14" />
        <span class="expr-text">{{ generatedExpr }}</span>
      </div>
      <div class="schedule-desc">{{ scheduleDescription }}</div>
    </div>

    <!-- 每天 Tab -->
    <div v-if="activeFreq === 'daily'" class="freq-content">
      <div class="config-row">
        <Clock class="config-icon" :size="16" />
        <span class="config-label">执行时间</span>
        <CompactSelect
          :model-value="dailyHour"
          :options="hourOptions"
          :disabled="disabled"
          min-width="56px"
          @update:model-value="v => dailyHour = v as number"
        />
        <span class="config-separator">:</span>
        <CompactSelect
          :model-value="dailyMinute"
          :options="minuteOptions"
          :disabled="disabled"
          min-width="56px"
          @update:model-value="v => dailyMinute = v as number"
        />
      </div>
      <div class="expr-preview">
        <Code class="expr-icon" :size="14" />
        <span class="expr-text">{{ generatedExpr }}</span>
      </div>
      <div class="schedule-desc">{{ scheduleDescription }}</div>
    </div>

    <!-- 每周 Tab -->
    <div v-if="activeFreq === 'weekly'" class="freq-content">
      <div class="config-row config-row-wrap">
        <span class="config-label">重复日期</span>
        <div class="weekday-chips">
          <button
            v-for="day in WEEKDAY_OPTIONS"
            :key="day.value"
            class="weekday-chip"
            :class="{ active: weeklyDays.includes(day.value) }"
            :disabled="disabled"
            @click="toggleWeekday(day.value)"
          >
            {{ day.label }}
          </button>
        </div>
      </div>
      <div class="config-row">
        <Clock class="config-icon" :size="16" />
        <span class="config-label">执行时间</span>
        <CompactSelect
          :model-value="weeklyHour"
          :options="hourOptions"
          :disabled="disabled"
          min-width="56px"
          @update:model-value="v => weeklyHour = v as number"
        />
        <span class="config-separator">:</span>
        <CompactSelect
          :model-value="weeklyMinute"
          :options="minuteOptions"
          :disabled="disabled"
          min-width="56px"
          @update:model-value="v => weeklyMinute = v as number"
        />
      </div>
      <div class="expr-preview">
        <Code class="expr-icon" :size="14" />
        <span class="expr-text">{{ generatedExpr }}</span>
      </div>
      <div class="schedule-desc">{{ scheduleDescription }}</div>
    </div>

    <!-- 每月 Tab -->
    <div v-if="activeFreq === 'monthly'" class="freq-content">
      <div class="config-row">
        <Calendar class="config-icon" :size="16" />
        <span class="config-label">每月</span>
        <CompactSelect
          :model-value="monthlyDay"
          :options="dayOptions"
          :disabled="disabled"
          min-width="100px"
          @update:model-value="v => monthlyDay = v as number | 'L'"
        />
        <Clock class="config-icon" :size="16" />
        <CompactSelect
          :model-value="monthlyHour"
          :options="hourOptions"
          :disabled="disabled"
          min-width="56px"
          @update:model-value="v => monthlyHour = v as number"
        />
        <span class="config-separator">:</span>
        <CompactSelect
          :model-value="monthlyMinute"
          :options="minuteOptions"
          :disabled="disabled"
          min-width="56px"
          @update:model-value="v => monthlyMinute = v as number"
        />
      </div>
      <div class="expr-preview">
        <Code class="expr-icon" :size="14" />
        <span class="expr-text">{{ generatedExpr }}</span>
      </div>
      <div class="schedule-desc">{{ scheduleDescription }}</div>
    </div>

    <!-- 自定义 Tab -->
    <div v-if="activeFreq === 'custom'" class="freq-content">
      <!-- 非标准格式提示 -->
      <div v-if="isNonStandard" class="non-standard-hint">
        <AlertTriangle class="hint-icon" :size="14" />
        <span>此任务使用非标准调度格式（如 "30m"），修改后将以标准 Cron 表达式保存</span>
      </div>

      <div class="custom-input-row">
        <span class="config-label">Cron 表达式</span>
        <input
          v-model="customExpr"
          type="text"
          class="custom-input"
          placeholder="0 9 * * *"
          :disabled="disabled"
        />
      </div>

      <!-- 验证状态 -->
      <div v-if="customExpr.trim()" class="validation-status" :class="{ valid: exprValid === true, invalid: exprValid === false }">
        <template v-if="exprValid === true">
          ✓ 表达式有效
        </template>
        <template v-else-if="exprValid === false">
          ✗ 表达式格式无效
        </template>
      </div>

      <!-- 描述 -->
      <div v-if="customExpr.trim() && exprValid" class="schedule-desc">
        {{ scheduleDescription }}
      </div>

      <!-- 快捷预设 -->
      <div class="quick-presets">
        <span class="presets-label">快捷选择</span>
        <div class="preset-chips">
          <button
            v-for="preset in QUICK_PRESETS"
            :key="preset.expr"
            class="preset-chip"
            :disabled="disabled"
            @click="applyPreset(preset.expr)"
          >
            {{ preset.label }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cron-schedule-editor {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.cron-schedule-editor.disabled {
  opacity: 0.6;
  pointer-events: none;
}

/* 频率 Tab */
.freq-tabs {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.freq-tab {
  padding: 6px 14px;
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-md);
  background: var(--oc-card);
  color: var(--oc-text-secondary);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--duration-200) var(--ease-smooth);
  user-select: none;
}

.freq-tab:hover {
  border-color: var(--primary-400);
  color: var(--oc-text-primary);
}

.freq-tab.active {
  background: color-mix(in srgb, var(--primary-500) 12%, transparent);
  border-color: var(--primary-500);
  color: var(--primary-600);
  font-weight: var(--font-weight-medium);
}

.freq-tab:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

/* 分割线 */
.tab-divider {
  height: 1px;
  background: var(--oc-divider);
  margin: 14px 0;
}

/* 频率内容 */
.freq-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 配置行 */
.config-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.config-row-wrap {
  flex-wrap: wrap;
}

.config-icon {
  color: var(--oc-text-muted);
  flex-shrink: 0;
}

.config-label {
  font-size: var(--text-sm);
  color: var(--oc-text-secondary);
  white-space: nowrap;
}

.config-separator {
  font-size: var(--text-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--oc-text-primary);
}

/* 星期多选 */
.weekday-chips {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.weekday-chip {
  padding: 5px 10px;
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-md);
  background: var(--oc-card);
  color: var(--oc-text-secondary);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--duration-200) var(--ease-smooth);
  user-select: none;
}

.weekday-chip:hover {
  border-color: var(--primary-400);
}

.weekday-chip.active {
  background: var(--primary-500);
  border-color: var(--primary-500);
  color: white;
  font-weight: var(--font-weight-medium);
}

.weekday-chip:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

/* 表达式预览 */
.expr-preview {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: color-mix(in srgb, var(--oc-bg-secondary) 60%, transparent);
  border-radius: var(--radius-sm);
}

.expr-icon {
  color: var(--oc-text-muted);
  flex-shrink: 0;
}

.expr-text {
  font-family: var(--font-mono, 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace);
  font-size: var(--text-xs);
  color: var(--oc-text-secondary);
  user-select: all;
}

/* 调度描述 */
.schedule-desc {
  font-size: var(--text-sm);
  color: var(--primary-600);
}

/* 非标准格式提示 */
.non-standard-hint {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 12px;
  background: color-mix(in srgb, var(--oc-warning) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--oc-warning) 30%, transparent);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  color: var(--oc-warning);
}

.hint-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

/* 自定义输入 */
.custom-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.custom-input {
  flex: 1;
  padding: 6px 10px;
  background: var(--oc-card);
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-md);
  color: var(--oc-text-primary);
  font-family: var(--font-mono, 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace);
  font-size: var(--text-sm);
  transition: all var(--duration-200) var(--ease-smooth);
}

.custom-input:focus {
  outline: none;
  border-color: var(--primary-500);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary-500) 15%, transparent);
}

.custom-input::placeholder {
  color: var(--oc-text-muted);
}

/* 验证状态 */
.validation-status {
  font-size: var(--text-xs);
  padding: 2px 0;
}

.validation-status.valid {
  color: var(--oc-success);
}

.validation-status.invalid {
  color: var(--oc-error);
}

/* 快捷预设 */
.quick-presets {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.presets-label {
  font-size: var(--text-xs);
  color: var(--oc-text-muted);
}

.preset-chips {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.preset-chip {
  padding: 3px 10px;
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-sm);
  background: var(--oc-card);
  color: var(--oc-text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all var(--duration-200) var(--ease-smooth);
}

.preset-chip:hover {
  border-color: var(--primary-400);
  color: var(--primary-600);
  background: color-mix(in srgb, var(--primary-500) 8%, transparent);
}

.preset-chip:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

@media (max-width: 640px) {
  .freq-tabs {
    gap: 3px;
  }

  .freq-tab {
    padding: 5px 10px;
    font-size: var(--text-xs);
  }

  .weekday-chips {
    gap: 3px;
  }

  .weekday-chip {
    padding: 4px 8px;
    font-size: var(--text-xs);
  }
}
</style>
