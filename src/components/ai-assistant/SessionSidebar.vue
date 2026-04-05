<template>
  <aside class="session-sidebar">
    <!-- 顶部：新会话按钮 -->
    <div class="session-sidebar__header">
      <button class="session-sidebar__new-btn" @click="$emit('create')">
        <Plus :size="15" />
        <span>新会话</span>
      </button>
    </div>

    <!-- 会话列表 -->
    <div class="session-sidebar__list">
      <SessionItem
        v-for="session in sortedSessions"
        :key="session.id"
        :session="session"
        :is-active="session.id === currentSessionId"
        @switch="(id) => $emit('switch', id)"
        @delete="(id) => $emit('delete', id)"
        @rename="(id, title) => $emit('rename', id, title)"
      />

      <!-- 空状态 -->
      <div v-if="sessions.length === 0" class="session-sidebar__empty">
        <MessageSquare :size="24" class="session-sidebar__empty-icon" />
        <span>暂无会话</span>
      </div>
    </div>

    <!-- 底部统计 -->
    <div v-if="sessions.length > 0" class="session-sidebar__footer">
      {{ sessions.length }} 个会话
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Plus, MessageSquare } from 'lucide-vue-next'
import SessionItem from './SessionItem.vue'
import type { SessionInfo } from '@/types/ai'

const props = defineProps<{
  sessions: SessionInfo[]
  currentSessionId: string
}>()

defineEmits<{
  create: []
  switch: [id: string]
  delete: [id: string]
  rename: [id: string, title: string]
}>()

// 按最近活跃时间排序
const sortedSessions = computed(() => {
  return [...props.sessions].sort((a, b) => {
    return (b.last_active || b.created_at) - (a.last_active || a.created_at)
  })
})
</script>

<style scoped>
.session-sidebar {
  display: flex;
  flex-direction: column;
  width: 220px;
  min-width: 220px;
  border-right: 1px solid var(--oc-divider);
  background: var(--bg-surface);
  flex-shrink: 0;
}

/* 头部 */
.session-sidebar__header {
  padding: 0.75rem;
  flex-shrink: 0;
}

.session-sidebar__new-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px dashed var(--oc-divider);
  background: transparent;
  border-radius: var(--radius-md, 8px);
  color: var(--oc-text-secondary);
  font-size: 0.8125rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.session-sidebar__new-btn:hover {
  border-color: var(--oc-accent, #0d9488);
  color: var(--oc-accent, #0d9488);
  background: color-mix(in srgb, var(--oc-accent, #0d9488) 8%, transparent);
}

/* 列表 */
.session-sidebar__list {
  flex: 1;
  overflow-y: auto;
  padding: 0 0.5rem;
  min-height: 0;
}

.session-sidebar__list::-webkit-scrollbar {
  width: 4px;
}

.session-sidebar__list::-webkit-scrollbar-track {
  background: transparent;
}

.session-sidebar__list::-webkit-scrollbar-thumb {
  background: var(--oc-divider);
  border-radius: 2px;
}

/* 空状态 */
.session-sidebar__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem 1rem;
  color: var(--oc-text-quiet);
  font-size: 0.8125rem;
}

.session-sidebar__empty-icon {
  opacity: 0.4;
}

/* 底部 */
.session-sidebar__footer {
  padding: 0.5rem 0.75rem;
  border-top: 1px solid var(--oc-divider);
  font-size: 0.6875rem;
  color: var(--oc-text-quiet);
  text-align: center;
  flex-shrink: 0;
}
</style>
