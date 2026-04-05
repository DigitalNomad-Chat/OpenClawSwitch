<template>
  <div
    :class="['session-item', { active: isActive }]"
    @click="$emit('switch', session.id)"
    @dblclick="startEditing"
  >
    <!-- 活跃指示线 -->
    <div v-if="isActive" class="session-item__indicator"></div>

    <!-- 图标 -->
    <MessageSquare class="session-item__icon" :size="15" />

    <!-- 标题（可编辑） -->
    <div class="session-item__title-wrap">
      <input
        v-if="isEditing"
        ref="inputRef"
        v-model="editTitle"
        class="session-item__edit-input"
        @blur="commitRename"
        @keydown.enter="commitRename"
        @keydown.escape="cancelEdit"
        @click.stop
      />
      <span v-else class="session-item__title" :title="session.title">
        {{ session.title || '新会话' }}
      </span>
    </div>

    <!-- 相对时间 -->
    <span class="session-item__time">{{ relativeTime }}</span>

    <!-- 操作按钮（hover 显示） -->
    <div class="session-item__actions" @click.stop>
      <button
        class="session-item__action-btn"
        data-tooltip="重命名"
        @click="startEditing"
      >
        <Pencil :size="13" />
      </button>
      <button
        class="session-item__action-btn session-item__action-btn--danger"
        data-tooltip="删除"
        @click="handleDelete"
      >
        <Trash2 :size="13" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { MessageSquare, Pencil, Trash2 } from 'lucide-vue-next'
import type { SessionInfo } from '@/types/ai'

const props = defineProps<{
  session: SessionInfo
  isActive: boolean
}>()

const emit = defineEmits<{
  switch: [id: string]
  delete: [id: string]
  rename: [id: string, title: string]
}>()

const isEditing = ref(false)
const editTitle = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const showDeleteConfirm = ref(false)

// 相对时间
const relativeTime = computed(() => {
  const ts = props.session.last_active || props.session.created_at
  if (!ts) return ''
  const diff = Date.now() - ts
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes}分钟前`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}小时前`
  const days = Math.floor(hours / 24)
  if (days === 1) return '昨天'
  if (days < 30) return `${days}天前`
  return new Date(ts).toLocaleDateString()
})

const startEditing = () => {
  editTitle.value = props.session.title || ''
  isEditing.value = true
  nextTick(() => {
    inputRef.value?.focus()
    inputRef.value?.select()
  })
}

const commitRename = () => {
  const trimmed = editTitle.value.trim()
  if (trimmed && trimmed !== props.session.title) {
    emit('rename', props.session.id, trimmed)
  }
  isEditing.value = false
}

const cancelEdit = () => {
  isEditing.value = false
}

const handleDelete = () => {
  if (showDeleteConfirm.value) {
    emit('delete', props.session.id)
    showDeleteConfirm.value = false
  } else {
    showDeleteConfirm.value = true
    setTimeout(() => {
      showDeleteConfirm.value = false
    }, 3000)
  }
}
</script>

<style scoped>
.session-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-md, 8px);
  cursor: pointer;
  transition: all 0.15s ease;
  position: relative;
  min-height: 36px;
}

.session-item:hover {
  background: var(--oc-item-hover);
}

.session-item.active {
  background: var(--oc-item-active);
}

/* 活跃指示线 */
.session-item__indicator {
  position: absolute;
  left: 0;
  top: 6px;
  bottom: 6px;
  width: 3px;
  border-radius: 2px;
  background: var(--oc-accent, #0d9488);
}

/* 图标 */
.session-item__icon {
  flex-shrink: 0;
  color: var(--oc-text-tertiary);
}

.session-item.active .session-item__icon {
  color: var(--oc-accent, #0d9488);
}

/* 标题 */
.session-item__title-wrap {
  flex: 1;
  min-width: 0;
}

.session-item__title {
  display: block;
  font-size: 0.8125rem;
  color: var(--oc-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.session-item__edit-input {
  width: 100%;
  font-size: 0.8125rem;
  color: var(--oc-text-primary);
  background: var(--bg-primary);
  border: 1px solid var(--oc-accent, #0d9488);
  border-radius: var(--radius-sm, 4px);
  padding: 0.1rem 0.375rem;
  outline: none;
  line-height: 1.4;
}

/* 时间 */
.session-item__time {
  flex-shrink: 0;
  font-size: 0.6875rem;
  color: var(--oc-text-quiet);
  white-space: nowrap;
}

/* 操作按钮 */
.session-item__actions {
  display: none;
  align-items: center;
  gap: 0.125rem;
  flex-shrink: 0;
}

.session-item:hover .session-item__actions {
  display: flex;
}

.session-item__action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  border-radius: var(--radius-sm, 4px);
  color: var(--oc-text-tertiary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.session-item__action-btn:hover {
  background: var(--oc-item-hover);
  color: var(--oc-text-primary);
}

.session-item__action-btn--danger:hover {
  background: color-mix(in srgb, var(--oc-danger, #ef4444) 15%, transparent);
  color: var(--oc-danger, #ef4444);
}
</style>
