// ============================================================================
// 标签操作 — 共享工厂函数
// Vue 3 模板会自动解包 ref，在模板表达式中传参拿到的不是 ref 对象
// 所以用工厂函数在 <script setup> 闭包中捕获 ref，模板只调用返回的 handler
// ============================================================================

import type { Ref } from 'vue'

export interface TagHandlers {
  push: () => void
  pop: (item: string) => void
  onKeydown: (e: KeyboardEvent) => void
}

/**
 * 为标签列表创建 push/pop/onKeydown handler
 * 闭包捕获 ref 对象，不受模板自动解包影响
 */
export function createTagHandlers(listRef: Ref<string[]>, inputRef: Ref<string>): TagHandlers {
  return {
    push() {
      const val = inputRef.value.trim()
      if (val && !listRef.value.includes(val)) {
        listRef.value.push(val)
      }
      inputRef.value = ''
    },
    pop(item: string) {
      const idx = listRef.value.indexOf(item)
      if (idx !== -1) {
        listRef.value.splice(idx, 1)
      }
    },
    onKeydown(e: KeyboardEvent) {
      if (e.key === 'Enter') {
        e.preventDefault()
        this.push()
      }
    },
  }
}
