<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { Play, Globe, Network } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import type { PingResult, DnsResult } from '@/types/remote'

const emit = defineEmits<{
  toast: [type: 'success' | 'error', message: string]
}>()

const pingHost = ref('8.8.8.8')
const pingLoading = ref(false)
const pingResult = ref<PingResult | null>(null)

const dnsDomain = ref('google.com')
const dnsLoading = ref(false)
const dnsResult = ref<DnsResult | null>(null)

async function runPing() {
  if (!pingHost.value.trim()) return
  pingLoading.value = true
  try {
    pingResult.value = await invoke<PingResult>('remote_ping', { host: pingHost.value.trim() })
  } catch (err) {
    emit('toast', 'error', `Ping 失败: ${err}`)
    pingResult.value = null
  } finally {
    pingLoading.value = false
  }
}

async function runDns() {
  if (!dnsDomain.value.trim()) return
  dnsLoading.value = true
  try {
    dnsResult.value = await invoke<DnsResult>('remote_dns_resolve', { domain: dnsDomain.value.trim() })
  } catch (err) {
    emit('toast', 'error', `DNS 解析失败: ${err}`)
    dnsResult.value = null
  } finally {
    dnsLoading.value = false
  }
}
</script>

<template>
  <div class="space-y-4">
    <!-- Ping -->
    <div class="p-3 rounded-lg" style="background: var(--oc-card-elevated);">
      <div class="flex items-center gap-2 mb-2">
        <Network class="w-4 h-4" style="color: var(--oc-accent);" />
        <span class="text-sm font-medium" style="color: var(--oc-text-primary);">Ping</span>
      </div>
      <div class="flex items-center gap-2">
        <Input v-model="pingHost" placeholder="输入 IP 或域名" class="h-8 text-xs flex-1" />
        <Button variant="outline" size="sm" :disabled="pingLoading" @click="runPing">
          <Play class="w-4 h-4" />
          执行
        </Button>
      </div>
      <div
        v-if="pingResult"
        class="mt-2 text-xs p-2 rounded font-mono whitespace-pre-wrap"
        :style="{
          background: pingResult.success ? 'var(--primary-50)' : 'rgba(239,68,68,0.08)',
          color: pingResult.success ? 'var(--primary-700)' : 'var(--oc-error)'
        }"
      >
        {{ pingResult.output }}
      </div>
    </div>

    <!-- DNS -->
    <div class="p-3 rounded-lg" style="background: var(--oc-card-elevated);">
      <div class="flex items-center gap-2 mb-2">
        <Globe class="w-4 h-4" style="color: var(--oc-accent);" />
        <span class="text-sm font-medium" style="color: var(--oc-text-primary);">DNS 解析</span>
      </div>
      <div class="flex items-center gap-2">
        <Input v-model="dnsDomain" placeholder="输入域名" class="h-8 text-xs flex-1" />
        <Button variant="outline" size="sm" :disabled="dnsLoading" @click="runDns">
          <Play class="w-4 h-4" />
          执行
        </Button>
      </div>
      <div
        v-if="dnsResult"
        class="mt-2 text-xs p-2 rounded"
        :style="{
          background: dnsResult.success ? 'var(--primary-50)' : 'rgba(239,68,68,0.08)',
          color: dnsResult.success ? 'var(--primary-700)' : 'var(--oc-error)'
        }"
      >
        <div v-if="dnsResult.records.length" class="space-y-1">
          <div v-for="record in dnsResult.records" :key="record" class="font-mono">
            {{ record }}
          </div>
        </div>
        <div v-else>未解析到记录</div>
      </div>
    </div>
  </div>
</template>
