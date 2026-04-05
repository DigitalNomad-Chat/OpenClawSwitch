<template>
  <div :class="['tool-approval-card', `risk-${approval.risk}`, { 'is-resolved': approval.status !== 'pending' }]">
    <div class="approval-header">
      <div class="approval-tool-icon">
        <ShieldAlert :size="16" />
      </div>
      <div class="approval-info">
        <span class="approval-tool-name">{{ getToolLabel(approval.tool) }}</span>
        <span :class="['approval-risk-badge', `badge-${approval.risk}`]">
          {{ getRiskLabel(approval.risk) }}
        </span>
      </div>
    </div>

    <div class="approval-input">
      <code>{{ approval.input || '无参数' }}</code>
    </div>

    <div v-if="approval.status === 'pending'" class="approval-actions">
      <button class="approval-btn reject-btn" @click="$emit('respond', approval.approvalId, false)">
        <X :size="14" />
        <span>拒绝</span>
      </button>
      <button class="approval-btn approve-btn" @click="$emit('respond', approval.approvalId, true)">
        <Check :size="14" />
        <span>允许执行</span>
      </button>
    </div>

    <div v-else class="approval-result">
      <div v-if="approval.status === 'approved'" class="result-approved">
        <CheckCircle2 :size="14" />
        <span>已允许</span>
      </div>
      <div v-else-if="approval.status === 'rejected'" class="result-rejected">
        <XCircle :size="14" />
        <span>已拒绝</span>
      </div>
      <div v-else-if="approval.status === 'timeout'" class="result-timeout">
        <Clock :size="14" />
        <span>已超时</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ShieldAlert, X, Check, CheckCircle2, XCircle, Clock } from 'lucide-vue-next'
import type { ToolApprovalState } from '@/types/ai'

interface Props {
  approval: ToolApprovalState
}

defineProps<Props>()

defineEmits<{
  respond: [approvalId: string, approved: boolean]
}>()

const getToolLabel = (tool: string): string => {
  const labels: Record<string, string> = {
    bash: 'Bash 命令',
    read: '读取文件',
    write: '写入文件',
    edit: '编辑文件',
    webfetch: '网络请求',
    openclaw_config: '配置操作',
  }
  return labels[tool] || tool
}

const getRiskLabel = (risk: string): string => {
  const labels: Record<string, string> = {
    low: '低风险',
    medium: '中风险',
    high: '高风险',
  }
  return labels[risk] || risk
}
</script>

<style scoped>
.tool-approval-card {
  padding: 0.625rem 0.75rem;
  border-radius: var(--radius-lg);
  border: 1px solid var(--oc-divider);
  background: var(--bg-surface-elevated);
  animation: approvalSlideIn 0.25s ease-out both;
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.tool-approval-card.is-resolved {
  opacity: 0.5;
  transform: scale(0.98);
}

/* 风险等级边框 */
.tool-approval-card.risk-low {
  border-left: 3px solid var(--oc-success);
}

.tool-approval-card.risk-medium {
  border-left: 3px solid var(--oc-warning);
}

.tool-approval-card.risk-high {
  border-left: 3px solid var(--oc-danger);
}

@keyframes approvalSlideIn {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Header */
.approval-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.375rem;
}

.approval-tool-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: var(--radius-md);
  background: var(--oc-item-hover);
  color: var(--oc-text-secondary);
  flex-shrink: 0;
}

.approval-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.approval-tool-name {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--oc-text-primary);
}

.approval-risk-badge {
  font-size: 0.6875rem;
  font-weight: 500;
  padding: 0.1rem 0.4rem;
  border-radius: 9999px;
}

.badge-low {
  background: color-mix(in srgb, var(--oc-success) 15%, transparent);
  color: var(--oc-success);
}

.badge-medium {
  background: color-mix(in srgb, var(--oc-warning) 15%, transparent);
  color: var(--oc-warning);
}

.badge-high {
  background: color-mix(in srgb, var(--oc-danger) 15%, transparent);
  color: var(--oc-danger);
}

/* Input */
.approval-input {
  margin-bottom: 0.5rem;
  padding: 0.375rem 0.5rem;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.approval-input code {
  font-family: 'JetBrains Mono', 'Menlo', 'Monaco', monospace;
  font-size: 0.75rem;
  color: var(--oc-text-secondary);
  word-break: break-all;
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Actions */
.approval-actions {
  display: flex;
  gap: 0.375rem;
  justify-content: flex-end;
}

.approval-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.3rem 0.625rem;
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--oc-text-secondary);
}

.approval-btn:hover {
  background: var(--oc-item-hover);
}

.approval-btn.reject-btn:hover {
  border-color: var(--oc-danger);
  color: var(--oc-danger);
}

.approval-btn.approve-btn {
  background: var(--primary-600);
  border-color: var(--primary-600);
  color: white;
}

.approval-btn.approve-btn:hover {
  opacity: 0.85;
}

/* Result */
.approval-result {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.75rem;
  justify-content: flex-end;
}

.result-approved {
  color: var(--oc-success);
}

.result-rejected {
  color: var(--oc-danger);
}

.result-timeout {
  color: var(--oc-warning);
}
</style>
