<template>
  <div class="ai-assistant-panel">
    <!-- 切换标签 -->
    <div class="panel-tabs">
      <button
        :class="['tab', { active: activeTab === 'chat' }]"
        @click="activeTab = 'chat'"
      >
        <MessageSquare :size="16" />
        对话
      </button>
      <button
        :class="['tab', { active: activeTab === 'config' }]"
        @click="activeTab = 'config'"
      >
        <Settings :size="16" />
        配置
      </button>
    </div>

    <!-- 内容区域 -->
    <div class="panel-content">
      <ChatArea v-if="activeTab === 'chat'" />
      <LlmConfigPanel v-else-if="activeTab === 'config'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { MessageSquare, Settings } from 'lucide-vue-next'
import ChatArea from './ChatArea.vue'
import LlmConfigPanel from './LlmConfigPanel.vue'

const activeTab = ref('chat')
</script>

<style scoped>
.ai-assistant-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.panel-tabs {
  display: flex;
  gap: 0.25rem;
  padding: 0.5rem 1rem;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
}

.tab {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border: none;
  background: transparent;
  border-radius: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.tab:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tab.active {
  background: var(--primary-bg);
  color: var(--primary);
}

.panel-content {
  flex: 1;
  overflow: hidden;
}

.panel-content > * {
  height: 100%;
}
</style>
