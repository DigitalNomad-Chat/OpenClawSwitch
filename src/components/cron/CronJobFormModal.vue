<script setup lang="ts">
import { ref, computed, watch, inject, type Ref, type ComputedRef } from 'vue'
import { X, HelpCircle } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import DeliveryTargetSelect from '@/components/ui/DeliveryTargetSelect.vue'
import StyledSelect from '@/components/ui/StyledSelect.vue'
import { validateCronExpression } from '@/utils/cronValidator'
import { parseCronExpression } from '@/utils/cronParser'
import { resolveDeliveryTarget, formatAgentDisplay } from '@/utils/bindingResolver'
import { createTimezoneOptions, createAgentOptions } from '@/utils/timezoneHelper'
import type { BindingInfo, AgentInfo } from '@/types/binding'
import type { CronJob } from '@/types/cron'
import type { CronJobFormData } from '@/composables/useCronJobs'

// ============================================================================
// Props & Emits
// ============================================================================

interface Props {
  show: boolean
  mode: 'create' | 'edit'
  job?: CronJob | null
  availableAgents?: string[]
}

const props = withDefaults(defineProps<Props>(), {
  availableAgents: () => []
})

const emit = defineEmits<{
  close: []
  save: [data: CronJobFormData]
}>()

// ============================================================================
// Inject bindings and agents from parent
// ============================================================================

// 从父组件注入 bindings 和 agents 数据
// provide 提供的是 ref/computed，直接使用
const bindings = inject<Ref<BindingInfo[]>>('cronBindings')
const agents = inject<Ref<AgentInfo[]>>('cronAgents')
const deliveryTargetOptions = inject<ComputedRef<Array<{ value: string; label: string; channel: string; peerId: string; peerKind: string; agentId?: string; agentName?: string }>>>('cronDeliveryTargetOptions')
const parseDeliveryTargetValue = inject<(value: string) => { channel: string; peerId: string }>('cronParseDeliveryTargetValue', (v: string) => {
  const i = v.indexOf(':')
  return i === -1 ? { channel: v, peerId: '' } : { channel: v.substring(0, i), peerId: v.substring(i + 1) }
})

// 安全访问
const bindingsValue = computed(() => bindings?.value ?? [])
const agentsValue = computed(() => agents?.value ?? [])

// 投递目标预览
const deliveryTargetPreview = computed(() => {
  const channel = formData.value.deliveryChannel || formData.value.deliveryMode === 'silent' ? undefined : undefined
  const to = formData.value.deliveryTarget || undefined
  if (!formData.value.deliveryChannel && !formData.value.deliveryTarget) {
    return ''
  }
  return resolveDeliveryTarget(
    formData.value.deliveryChannel || undefined,
    formData.value.deliveryTarget || undefined,
    bindingsValue.value,
    agentsValue.value
  )
})

// ============================================================================
// 常量定义
// ============================================================================

const TIMEZONES = [
  'Asia/Shanghai',
  'Asia/Hong_Kong',
  'Asia/Taipei',
  'Asia/Tokyo',
  'Asia/Seoul',
  'Asia/Singapore',
  'UTC',
  'America/New_York',
  'America/Los_Angeles',
  'Europe/London',
  'Europe/Paris',
  'Europe/Berlin'
]

// 时区选项（带图标和偏移）
const timezoneOptions = computed(() => createTimezoneOptions(TIMEZONES))

// Agent 选项（带图标）
const agentOptions = computed(() => {
  // 优先使用 inject 的 agents 数据（有完整信息）
  if (agentsValue.value.length > 0) {
    return createAgentOptions(agentsValue.value)
  }

  // 降级：使用 availableAgents prop
  if (props.availableAgents && props.availableAgents.length > 0) {
    // 如果是对象数组（新格式）
    if (typeof props.availableAgents[0] === 'object') {
      return createAgentOptions(props.availableAgents as AgentInfo[])
    }
    // 如果是字符串数组（旧格式）
    return createAgentOptions((props.availableAgents as string[]).map(id => ({ id })))
  }

  return []
})

const SESSION_TARGETS = [
  { value: 'isolated', label: '独立会话 (Isolated)', description: '每次执行创建新的独立会话' },
  { value: 'shared', label: '共享会话 (Shared)', description: '使用共享会话执行任务' },
  { value: 'persistent', label: '持久会话 (Persistent)', description: '使用持久化会话执行任务' }
] as const

const WAKE_MODES = [
  { value: 'now', label: '立即执行 (Now)', description: '到达调度时间立即执行' },
  { value: 'drift', label: '延迟执行 (Drift)', description: '延迟到下次可用时间执行' }
] as const

// ============================================================================
// 表单状态
// ============================================================================

const formData = ref<CronJobFormData>({
  name: '',
  description: '',
  agentId: '',
  cronExpr: '',
  timezone: 'Asia/Shanghai',
  message: '',
  deliveryMode: 'announce',
  deliveryChannel: '',
  deliveryTarget: '',
  sessionTarget: 'isolated',
  wakeMode: 'now'
})

const formErrors = ref<Partial<Record<keyof CronJobFormData, string>>>({})
const touched = ref<Partial<Record<keyof CronJobFormData, boolean>>>({})

// ============================================================================
// 计算属性
// ============================================================================

const cronExprValid = computed(() => {
  const expr = formData.value.cronExpr.trim()
  if (!expr) return null
  return validateCronExpression(expr)
})

const cronExprDescription = computed(() => {
  if (cronExprValid.value) {
    return parseCronExpression(formData.value.cronExpr)
  }
  return null
})

// 收集所有验证错误，用于显示给用户
const validationErrors = computed(() => {
  const errors: string[] = []

  if (!formData.value.name.trim()) {
    errors.push('任务名称不能为空')
  }
  if (!formData.value.agentId.trim()) {
    errors.push('请选择执行任务的 Agent')
  }
  if (!formData.value.cronExpr.trim()) {
    errors.push('Cron 表达式不能为空')
  } else if (!cronExprValid.value) {
    errors.push('Cron 表达式格式无效')
  }
  if (!formData.value.message.trim()) {
    errors.push('消息内容不能为空')
  }
  if (formData.value.deliveryMode === 'announce') {
    if (!formData.value.deliveryChannel || !formData.value.deliveryTarget) {
      errors.push('请选择投递目标（投递模式为"投递消息"时必须选择）')
    }
  }

  return errors
})

const canSave = computed(() => {
  // 必填字段检查
  if (!formData.value.name.trim()) {
    console.log('[CronJobFormModal] canSave = false: 任务名称为空')
    return false
  }
  if (!formData.value.agentId.trim()) {
    console.log('[CronJobFormModal] canSave = false: Agent为空')
    return false
  }
  if (!formData.value.cronExpr.trim()) {
    console.log('[CronJobFormModal] canSave = false: Cron表达式为空')
    return false
  }
  if (!cronExprValid.value) {
    console.log('[CronJobFormModal] canSave = false: Cron表达式无效')
    return false
  }
  if (!formData.value.message.trim()) {
    console.log('[CronJobFormModal] canSave = false: 消息为空')
    return false
  }

  // 投递模式检查（announce 模式必须选择投递目标）
  // 注意：检查 formData 中的实际数据，而不是 UI 组件的状态
  if (formData.value.deliveryMode === 'announce') {
    if (!formData.value.deliveryChannel || !formData.value.deliveryTarget) {
      console.log('[CronJobFormModal] canSave = false: 投递目标未选择', {
        channel: formData.value.deliveryChannel,
        target: formData.value.deliveryTarget
      })
      return false
    }
  }

  console.log('[CronJobFormModal] canSave = true')
  return true
})

const modalTitle = computed(() => {
  return props.mode === 'create' ? '新建定时任务' : '编辑定时任务'
})

// ============================================================================
// 表单验证
// ============================================================================

function validateField(field: keyof CronJobFormData) {
  touched.value[field] = true

  const value = formData.value[field]

  switch (field) {
    case 'name':
      if (!value?.trim()) {
        formErrors.value.name = '任务名称不能为空'
        return false
      }
      if (value.length > 100) {
        formErrors.value.name = '任务名称不能超过100个字符'
        return false
      }
      break

    case 'agentId':
      if (!value?.trim()) {
        formErrors.value.agentId = '请选择 Agent'
        return false
      }
      break

    case 'cronExpr':
      if (!value?.trim()) {
        formErrors.value.cronExpr = 'Cron 表达式不能为空'
        return false
      }
      if (!cronExprValid.value) {
        formErrors.value.cronExpr = '无效的 Cron 表达式格式'
        return false
      }
      break

    case 'message':
      if (!value?.trim()) {
        formErrors.value.message = '消息内容不能为空'
        return false
      }
      break

    case 'deliveryTarget':
      if (formData.value.deliveryMode === 'announce' && !value?.trim()) {
        formErrors.value.deliveryTarget = '请填写投递目标'
        return false
      }
      break
  }

  delete formErrors.value[field]
  return true
}

function validateAll() {
  let valid = true

  const fields: (keyof CronJobFormData)[] = ['name', 'agentId', 'cronExpr', 'message']
  if (formData.value.deliveryMode === 'announce') {
    fields.push('deliveryTarget')
  }

  for (const field of fields) {
    if (!validateField(field)) {
      valid = false
    }
  }

  return valid
}

// ============================================================================
// 操作处理
// ============================================================================

function handleClose() {
  emit('close')
}

function handleSave() {
  if (!validateAll()) {
    return
  }
  emit('save', formData.value)
}

function handleDeliveryModeChange(mode: 'announce' | 'silent') {
  formData.value.deliveryMode = mode
  if (mode === 'silent') {
    formData.value.deliveryChannel = ''
    formData.value.deliveryTarget = ''
  }
}

// 投递目标下拉选择的值
const deliveryTargetSelectValue = ref('')

function handleDeliveryTargetChange(selectedValue: string) {
  console.log('[CronJobFormModal] 投递目标变化:', selectedValue)
  if (!selectedValue) {
    formData.value.deliveryChannel = ''
    formData.value.deliveryTarget = ''
    console.log('[CronJobFormModal] 投递目标已清空')
    return
  }
  const parsed = parseDeliveryTargetValue(selectedValue)
  console.log('[CronJobFormModal] 解析结果:', parsed)
  formData.value.deliveryChannel = parsed.channel
  formData.value.deliveryTarget = parsed.peerId
  console.log('[CronJobFormModal] formData 更新后:', {
    channel: formData.value.deliveryChannel,
    target: formData.value.deliveryTarget
  })
}

// 编辑模式下回填投递目标选择值
function restoreDeliveryTargetSelectValue() {
  // 如果没有投递目标，清空选择值
  if (!formData.value.deliveryChannel || !formData.value.deliveryTarget) {
    deliveryTargetSelectValue.value = ''
    return
  }

  // 如果选项列表还没有加载，暂时不设置（等待选项加载完成后再次调用）
  if (!deliveryTargetOptions.value || deliveryTargetOptions.value.length === 0) {
    console.log('[CronJobFormModal] 选项列表未加载，延迟回填投递目标')
    deliveryTargetSelectValue.value = ''
    return
  }

  // 在选项列表中查找匹配的选项
  const targetValue = `${formData.value.deliveryChannel}:${formData.value.deliveryTarget}`
  const matchedOption = deliveryTargetOptions.value.find(opt => opt.value === targetValue)

  if (matchedOption) {
    // 找到匹配的选项，使用选项的 value（确保格式一致）
    deliveryTargetSelectValue.value = matchedOption.value
    console.log('[CronJobFormModal] 投递目标回填成功:', matchedOption.value)
  } else {
    // 原始选择的投递目标不在当前选项列表中（可能被删除了）
    console.warn('[CronJobFormModal] 原始投递目标不在选项列表中:', targetValue)
    deliveryTargetSelectValue.value = ''
  }
}

// ============================================================================
// 监听 props 变化
// ============================================================================

watch(
  () => props.show,
  (show) => {
    console.log('[CronJobFormModal] Modal show 状态变化:', show, 'mode:', props.mode)

    if (!show) {
      // Modal 关闭时重置表单
      console.log('[CronJobFormModal] Modal 关闭，重置表单')
      formData.value = {
        name: '',
        description: '',
        agentId: '',
        cronExpr: '',
        timezone: 'Asia/Shanghai',
        message: '',
        deliveryMode: 'announce',
        deliveryChannel: '',
        deliveryTarget: '',
        sessionTarget: 'isolated',
        wakeMode: 'now'
      }
      formErrors.value = {}
      touched.value = {}
      deliveryTargetSelectValue.value = ''
      return
    }

    // 编辑模式:填充现有数据
    if (props.mode === 'edit' && props.job) {
      const job = props.job
      console.log('[CronJobFormModal] 编辑模式，填充数据:', job)
      formData.value = {
        name: job.name,
        description: job.description || '',
        agentId: job.agentId || '',
        cronExpr: job.schedule.expr,
        timezone: job.schedule.tz,
        message: job.payload.message || '',
        deliveryMode: job.delivery?.mode || 'announce',
        deliveryChannel: job.delivery?.channel || '',
        deliveryTarget: job.delivery?.to || '',
        sessionTarget: job.sessionTarget as any,
        wakeMode: job.wakeMode as any
      }
      console.log('[CronJobFormModal] formData 已填充:', {
        deliveryChannel: formData.value.deliveryChannel,
        deliveryTarget: formData.value.deliveryTarget
      })
      // 回填投递目标选择值
      restoreDeliveryTargetSelectValue()
    } else {
      // 新建模式：重置选择值
      console.log('[CronJobFormModal] 新建模式，重置选择值')
      deliveryTargetSelectValue.value = ''
    }
  },
  { immediate: true }
)

// 监听投递目标选项列表加载完成
// 当选项列表准备好后，如果当前是编辑模式且有投递目标，则回填
watch(
  () => deliveryTargetOptions.value,
  (options) => {
    // 选项列表加载完成后，如果当前有投递目标需要回填
    if (options.length > 0 && formData.value.deliveryChannel && formData.value.deliveryTarget) {
      restoreDeliveryTargetSelectValue()
    }
  },
  { immediate: true }
)
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="handleClose">
    <div class="modal-content oc-panel" @click.stop>
      <!-- 头部 -->
      <div class="modal-header">
        <h3 class="modal-title">{{ modalTitle }}</h3>
        <Button variant="ghost" size="sm" @click="handleClose">
          <X class="w-4 h-4" />
        </Button>
      </div>

      <!-- 表单内容 -->
      <div class="modal-body">
        <!-- 验证错误提示 -->
        <Transition name="error-alert">
          <div v-if="validationErrors.length > 0" class="validation-error-alert">
            <div class="error-alert-icon">⚠️</div>
            <div class="error-alert-content">
              <div class="error-alert-title">无法保存</div>
              <ul class="error-alert-list">
                <li v-for="(error, index) in validationErrors" :key="index">{{ error }}</li>
              </ul>
            </div>
          </div>
        </Transition>

        <!-- 基本信息 -->
        <section class="form-section">
          <h4 class="section-title">基本信息</h4>

          <!-- 任务名称 -->
          <div class="form-field">
            <label class="field-label">
              任务名称 <span class="required">*</span>
            </label>
            <input
              v-model="formData.name"
              type="text"
              class="field-input"
              placeholder="例如: 每日天气播报"
              @blur="validateField('name')"
            />
            <span v-if="formErrors.name" class="field-error">{{ formErrors.name }}</span>
          </div>

          <!-- 任务描述 -->
          <div class="form-field">
            <label class="field-label">任务描述</label>
            <textarea
              v-model="formData.description"
              class="field-textarea"
              rows="2"
              placeholder="简要描述此定时任务的用途..."
            />
          </div>

          <!-- Agent ID -->
          <div class="form-field">
            <label class="field-label">
              Agent <span class="required">*</span>
            </label>
            <StyledSelect
              v-model="formData.agentId"
              :options="agentOptions"
              placeholder="请选择 Agent"
              :error="!!formErrors.agentId"
              @blur="validateField('agentId')"
            />
            <span v-if="formErrors.agentId" class="field-error">{{ formErrors.agentId }}</span>
          </div>
        </section>

        <!-- 调度配置 -->
        <section class="form-section">
          <h4 class="section-title">调度配置</h4>

          <!-- Cron 表达式 -->
          <div class="form-field">
            <label class="field-label">
              Cron 表达式 <span class="required">*</span>
              <HelpCircle class="inline-help" title="格式: 分 时 日 月 周 (例如: 0 9 * * * 表示每天9点)" />
            </label>
            <input
              v-model="formData.cronExpr"
              type="text"
              class="field-input"
              placeholder="0 9 * * *"
              @blur="validateField('cronExpr')"
            />
            <div v-if="cronExprDescription" class="cron-description">
              {{ cronExprDescription }}
            </div>
            <span v-if="formErrors.cronExpr" class="field-error">{{ formErrors.cronExpr }}</span>
          </div>

          <!-- 时区 -->
          <div class="form-field">
            <label class="field-label">时区</label>
            <StyledSelect
              v-model="formData.timezone"
              :options="timezoneOptions"
              placeholder="请选择时区"
            />
          </div>
        </section>

        <!-- 消息内容 -->
        <section class="form-section">
          <h4 class="section-title">消息内容</h4>

          <div class="form-field">
            <label class="field-label">
              消息 <span class="required">*</span>
            </label>
            <textarea
              v-model="formData.message"
              class="field-textarea"
              rows="4"
              placeholder="输入定时任务要发送的消息内容..."
              @blur="validateField('message')"
            />
            <span v-if="formErrors.message" class="field-error">{{ formErrors.message }}</span>
          </div>
        </section>

        <!-- 投递配置 -->
        <section class="form-section">
          <h4 class="section-title">投递配置</h4>

          <!-- 投递模式 -->
          <div class="form-field">
            <label class="field-label">投递模式</label>
            <div class="radio-group">
              <label class="radio-option">
                <input
                  type="radio"
                  :value="'announce'"
                  v-model="formData.deliveryMode"
                  @change="handleDeliveryModeChange('announce')"
                />
                <span>投递消息</span>
              </label>
              <label class="radio-option">
                <input
                  type="radio"
                  :value="'silent'"
                  v-model="formData.deliveryMode"
                  @change="handleDeliveryModeChange('silent')"
                />
                <span>静默模式</span>
              </label>
            </div>
          </div>

          <!-- 投递目标（从 bindings 生成下拉列表） -->
          <template v-if="formData.deliveryMode === 'announce'">
            <div class="form-field">
              <label class="field-label">
                投递目标 <span class="required">*</span>
              </label>
              <DeliveryTargetSelect
                v-model="deliveryTargetSelectValue"
                :options="deliveryTargetOptions"
                placeholder="请选择投递目标"
                :error="!!formErrors.deliveryTarget"
                @change="handleDeliveryTargetChange"
              />
              <span v-if="formErrors.deliveryTarget" class="field-error">{{ formErrors.deliveryTarget }}</span>
            </div>
          </template>
        </section>

        <!-- 高级配置 -->
        <section class="form-section">
          <h4 class="section-title">高级配置</h4>

          <!-- Session 模式 -->
          <div class="form-field">
            <label class="field-label">Session 模式</label>
            <div class="option-cards">
              <label
                v-for="option in SESSION_TARGETS"
                :key="option.value"
                class="option-card"
                :class="{ active: formData.sessionTarget === option.value }"
              >
                <input type="radio" :value="option.value" v-model="formData.sessionTarget" />
                <div class="option-content">
                  <div class="option-label">{{ option.label }}</div>
                  <div class="option-description">{{ option.description }}</div>
                </div>
              </label>
            </div>
          </div>

          <!-- Wake 模式 -->
          <div class="form-field">
            <label class="field-label">唤醒模式</label>
            <div class="option-cards">
              <label
                v-for="option in WAKE_MODES"
                :key="option.value"
                class="option-card"
                :class="{ active: formData.wakeMode === option.value }"
              >
                <input type="radio" :value="option.value" v-model="formData.wakeMode" />
                <div class="option-content">
                  <div class="option-label">{{ option.label }}</div>
                  <div class="option-description">{{ option.description }}</div>
                </div>
              </label>
            </div>
          </div>
        </section>
      </div>

      <!-- 底部操作栏 -->
      <div class="modal-footer">
        <Button variant="outline" @click="handleClose">取消</Button>
        <Button variant="default" :disabled="!canSave" @click="handleSave">
          {{ mode === 'create' ? '创建任务' : '保存修改' }}
        </Button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
  backdrop-filter: blur(4px);
}

.modal-content {
  width: 100%;
  max-width: 640px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-xl);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--oc-divider);
}

.modal-title {
  font-size: var(--text-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--oc-text-primary);
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

/* 验证错误提示 */
.validation-error-alert {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  margin-bottom: 20px;
  background: color-mix(in srgb, var(--oc-danger) 8%, transparent);
  border: 1px solid var(--oc-danger);
  border-left: 4px solid var(--oc-danger);
  border-radius: var(--radius-md);
}

.error-alert-icon {
  font-size: 20px;
  line-height: 1;
  flex-shrink: 0;
}

.error-alert-content {
  flex: 1;
  min-width: 0;
}

.error-alert-title {
  font-size: var(--text-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--oc-danger);
  margin-bottom: 6px;
}

.error-alert-list {
  margin: 0;
  padding-left: 18px;
}

.error-alert-list li {
  font-size: var(--text-sm);
  color: var(--oc-text-secondary);
  line-height: 1.5;
  margin-bottom: 2px;
}

.error-alert-list li:last-child {
  margin-bottom: 0;
}

/* 错误提示动画 */
.error-alert-enter-active,
.error-alert-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.error-alert-enter-from,
.error-alert-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--oc-divider);
  background: var(--oc-card-elevated);
}

.form-section {
  margin-bottom: 24px;
}

.form-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: var(--text-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--oc-text-primary);
  margin-bottom: 16px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.form-field {
  margin-bottom: 16px;
}

.field-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: var(--text-sm);
  font-weight: var(--font-weight-medium);
  color: var(--oc-text-secondary);
  margin-bottom: 6px;
}

.required {
  color: var(--oc-error);
}

.inline-help {
  width: 14px;
  height: 14px;
  color: var(--oc-text-muted);
  cursor: help;
  flex-shrink: 0;
}

.field-input,
.field-select,
.field-textarea {
  width: 100%;
  padding: 10px 12px;
  background: var(--oc-card);
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-md);
  color: var(--oc-text-primary);
  font-size: var(--text-sm);
  transition: all var(--duration-200) var(--ease-smooth);
}

.field-input:focus,
.field-select:focus,
.field-textarea:focus {
  outline: none;
  border-color: var(--primary-500);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary-500) 15%, transparent);
}

.field-input::placeholder,
.field-textarea::placeholder {
  color: var(--oc-text-muted);
}

.field-textarea {
  resize: vertical;
  min-height: 80px;
  font-family: inherit;
  line-height: 1.5;
}

.field-error {
  display: block;
  margin-top: 4px;
  font-size: var(--text-xs);
  color: var(--oc-error);
}

.field-hint {
  margin-top: 6px;
  font-size: var(--text-xs);
  color: var(--oc-text-muted);
}

.cron-description {
  margin-top: 6px;
  padding: 8px 12px;
  background: color-mix(in srgb, var(--primary-500) 10%, transparent);
  border-left: 3px solid var(--primary-500);
  border-radius: var(--radius-sm);
  font-size: var(--text-sm);
  color: var(--primary-700);
}

.radio-group {
  display: flex;
  gap: 16px;
}

.radio-option {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: var(--text-sm);
  color: var(--oc-text-secondary);
}

.radio-option input[type="radio"] {
  width: 16px;
  height: 16px;
  accent-color: var(--primary-500);
}

.option-cards {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.option-card {
  display: flex;
  gap: 12px;
  padding: 12px;
  background: var(--oc-card);
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-200) var(--ease-smooth);
}

.option-card:hover {
  border-color: var(--primary-400);
}

.option-card.active {
  border-color: var(--primary-500);
  background: color-mix(in srgb, var(--primary-500) 8%, transparent);
}

.option-card input[type="radio"] {
  display: none;
}

.option-content {
  flex: 1;
}

.option-label {
  font-size: var(--text-sm);
  font-weight: var(--font-weight-medium);
  color: var(--oc-text-primary);
}

.option-description {
  margin-top: 2px;
  font-size: var(--text-xs);
  color: var(--oc-text-muted);
}

@media (max-width: 640px) {
  .modal-content {
    max-height: 95vh;
  }

  .modal-header,
  .modal-body,
  .modal-footer {
    padding-left: 16px;
    padding-right: 16px;
  }
}
</style>
