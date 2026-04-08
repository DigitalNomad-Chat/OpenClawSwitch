<script setup lang="ts">
import { ref } from 'vue'
import { cn } from '@/lib/utils'
import { Trash2, Star, Shield, ChevronDown, ChevronUp, Cpu, Brain, Zap, Plus, X, Pencil, Server, Clipboard } from 'lucide-vue-next'
import Button from './ui/Button.vue'
import type { ProviderInfo, ModelInfo } from '@/types/config'

interface Props {
  provider: ProviderInfo
  containsPrimary?: boolean
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  containsPrimary: false
})

const emit = defineEmits<{
  setPrimary: [modelPath: string]
  setFallback: [modelPath: string]
  addModel: [providerName: string]
  removeModel: [providerName: string, modelId: string]
  edit: []
  delete: []
  copyCommand: [command: string]
}>()

const showModels = ref(false)

const handleRemoveModel = (modelId: string) => {
  emit('removeModel', props.provider.name, modelId)
}

const formatContextWindow = (value?: number): string => {
  if (!value) return ''
  if (value >= 1000000) return `${(value / 1000000).toFixed(1)}M`
  if (value >= 1000) return `${(value / 1000).toFixed(0)}K`
  return value.toString()
}

const handleSetPrimary = (model?: ModelInfo) => {
  const modelId = model?.id || 'default'
  emit('setPrimary', `${props.provider.name}/${modelId}`)
}

const handleSetFallback = (model?: ModelInfo) => {
  const modelId = model?.id || 'default'
  emit('setFallback', `${props.provider.name}/${modelId}`)
}

const handleCopySwitch = async (model?: ModelInfo) => {
  const modelId = model?.id || 'default'
  const command = `/model ${props.provider.name}/${modelId}`
  try {
    await navigator.clipboard.writeText(command)
    emit('copyCommand', command)
  } catch {
    // fallback: textarea copy
    const ta = document.createElement('textarea')
    ta.value = command
    ta.style.position = 'fixed'
    ta.style.opacity = '0'
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    emit('copyCommand', command)
  }
}
</script>

<template>
  <div
    :class="cn('oc-subpanel oc-provider-card p-4 transition-all', containsPrimary ? 'oc-provider-card-active' : '', props.class)"
  >
    <div class="flex items-start justify-between gap-3">
      <div class="flex-1 min-w-0">
        <!-- 服务商图标 + 名称 -->
        <div class="flex items-center gap-2.5 mb-2">
          <div class="provider-icon">
            <span class="provider-initial">{{ provider.name.charAt(0).toUpperCase() }}</span>
          </div>

          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <h3 class="text-sm font-semibold truncate" style="color: var(--oc-text-primary);">
                {{ provider.name }}
              </h3>
              <Star v-if="containsPrimary" class="w-3.5 h-3.5 flex-shrink-0" style="color: var(--oc-accent);" />
            </div>
            <p class="text-xs truncate mt-0.5" style="color: var(--oc-text-muted);">
              {{ provider.baseUrl }}
            </p>
          </div>
        </div>

        <!-- 状态信息 -->
        <div class="flex items-center gap-2 text-xs ml-[calc(36px+10px)]" style="color: var(--oc-text-muted);">
          <span :style="{ color: provider.hasApiKey ? 'var(--oc-success)' : 'var(--oc-text-quiet)' }">
            {{ provider.hasApiKey ? '✓' : '○' }} Key
          </span>
          <span style="color: var(--oc-divider-soft);">·</span>
          <span>{{ provider.api || 'API' }}</span>
        </div>

        <!-- 模型折叠按钮 -->
        <button @click="showModels = !showModels"
                class="flex items-center gap-1.5 text-xs mt-2 transition-colors ml-[calc(36px+10px)]"
                style="color: var(--oc-text-secondary);">
          <Server class="w-3.5 h-3.5" />
          <span>{{ provider.modelCount }} 个模型</span>
          <component :is="showModels ? ChevronUp : ChevronDown" class="w-3 h-3" />
        </button>

          <div v-if="showModels" class="mt-2 space-y-1 pl-2 border-l-2" style="border-color: var(--oc-divider);">
            <div v-for="model in provider.models" :key="model.id"
                 class="group flex items-center justify-between gap-2 rounded-[9px] border px-2 py-1.5 text-xs"
                 style="border-color: var(--oc-divider-soft); background: color-mix(in srgb, var(--oc-card-elevated) 82%, transparent);">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1">
                  <span class="truncate font-medium" style="color: var(--oc-text-primary);">{{ model.name || model.id }}</span>
                  <Brain v-if="model.reasoning" class="w-3 h-3 flex-shrink-0" style="color: var(--oc-accent);" />
                  <span v-if="model.contextWindow" class="flex flex-shrink-0 items-center gap-0.5" style="color: var(--oc-text-muted);">
                    <Zap class="w-2.5 h-2.5" />{{ formatContextWindow(model.contextWindow) }}
                  </span>
                </div>
                <div class="truncate" style="color: var(--oc-text-muted);">{{ model.id }}</div>
              </div>
              <div class="flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                <span class="oc-tooltip-wrap">
                  <Button variant="ghost" size="sm" @click="handleSetPrimary(model)" class="h-6 w-6 p-0">
                    <Star class="w-3 h-3" />
                  </Button>
                  <span class="oc-tooltip">设为主要模型</span>
                </span>
                <span class="oc-tooltip-wrap">
                  <Button variant="ghost" size="sm" @click="handleSetFallback(model)" class="h-6 w-6 p-0">
                    <Shield class="w-3 h-3" />
                  </Button>
                  <span class="oc-tooltip">设为备用模型</span>
                </span>
                <span class="oc-tooltip-wrap">
                  <Button variant="ghost" size="sm" @click="handleCopySwitch(model)" class="h-6 w-6 p-0">
                    <Clipboard class="w-3 h-3" />
                  </Button>
                  <span class="oc-tooltip">复制 /model 切换命令</span>
                </span>
                <span class="oc-tooltip-wrap">
                  <Button variant="ghost" size="sm" @click="handleRemoveModel(model.id)" class="h-6 w-6 p-0" style="color: var(--oc-danger);">
                    <X class="w-3 h-3" />
                  </Button>
                  <span class="oc-tooltip">删除模型</span>
                </span>
              </div>
            </div>

            <button @click="emit('addModel', provider.name)"
                    class="flex w-full items-center gap-1 px-2 py-1 text-left text-xs transition-colors"
                    style="color: var(--oc-text-secondary);">
              <Plus class="w-3 h-3" />
              添加模型
            </button>
          </div>
      </div>

      <div class="flex flex-col gap-1 flex-shrink-0">
        <Button variant="ghost" size="sm" @click="emit('edit')" class="h-7 text-xs" title="编辑">
          <Pencil class="w-3 h-3" />
        </Button>
        <Button variant="ghost" size="sm" @click="emit('delete')" class="h-7 text-xs" style="color: var(--oc-danger);" title="删除">
          <Trash2 class="w-3 h-3" />
        </Button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.oc-provider-card {
  overflow: visible;
  box-shadow: none;
}

.oc-provider-card-active {
  border-color: color-mix(in srgb, var(--oc-input-focus) 78%, var(--oc-card-border) 22%);
  box-shadow: none;
}

/* 首字母图标容器 */
.provider-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: linear-gradient(135deg,
    var(--primary-500) 0%,
    var(--primary-600) 100%
  );
  box-shadow:
    0 2px 8px color-mix(in srgb, var(--primary-500) 35%, transparent),
    inset 0 1px 0 rgba(255, 255, 255, 0.15);
  flex-shrink: 0;
}

.provider-initial {
  font-size: 15px;
  font-weight: 700;
  color: #ffffff;
  letter-spacing: -0.5px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

/* 暗黑模式适配 */
:root[data-theme='dark'] .provider-icon {
  background: linear-gradient(135deg,
    color-mix(in srgb, var(--primary-500) 90%, white) 0%,
    color-mix(in srgb, var(--primary-600) 90%, white) 100%
  );
  box-shadow:
    0 2px 8px color-mix(in srgb, var(--primary-500) 25%, transparent),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

/* 悬停效果 */
.oc-provider-card:hover .provider-icon {
  box-shadow:
    0 4px 12px color-mix(in srgb, var(--primary-500) 45%, transparent),
    inset 0 1px 0 rgba(255, 255, 255, 0.15);
}

/* 自定义 Tooltip */
.oc-tooltip-wrap {
  position: relative;
  display: inline-flex;
}

.oc-tooltip {
  position: absolute;
  bottom: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 11px;
  line-height: 1.4;
  white-space: nowrap;
  color: var(--oc-text-primary);
  background: var(--oc-card);
  border: 1px solid var(--oc-divider);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.15s ease;
  z-index: 30;
}

.oc-tooltip-wrap:hover .oc-tooltip {
  opacity: 1;
}
</style>
