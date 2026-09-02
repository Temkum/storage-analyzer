<script setup lang="ts">
import { computed } from 'vue'

import type { NetworkLive } from '@/types/network'
import { formatBytes } from '@/utils/format'

const props = defineProps<{
  live: NetworkLive | null
}>()

interface InterfaceTotal {
  interfaceId: string
  bytesReceived: number
  bytesSent: number
}

/** Totals over the retained 10-minute live window. */
const totals = computed<InterfaceTotal[]>(() => {
  const live = props.live

  if (!live) {
    return []
  }

  return live.interfaces.map((series) => ({
    interfaceId: series.interfaceId,
    bytesReceived: series.samples.reduce((sum, sample) => sum + sample.bytesReceived, 0),
    bytesSent: series.samples.reduce((sum, sample) => sum + sample.bytesSent, 0),
  }))
})
</script>

<template>
  <section class="interfaces">
    <div class="interfaces__header">
      <div>
        <h2>Interfaces</h2>
        <p>Totals across the retained 10-minute live window.</p>
      </div>

      <span class="interfaces__count">{{ totals.length }}</span>
    </div>

    <p v-if="totals.length === 0" class="interfaces__empty">No interface traffic observed yet.</p>

    <ul v-else class="interfaces__list">
      <li v-for="total in totals" :key="total.interfaceId" class="interfaces__item">
        <strong class="interfaces__name" :title="total.interfaceId">{{ total.interfaceId }}</strong>
        <span class="interfaces__rx">{{ formatBytes(total.bytesReceived) }} RX</span>
        <span class="interfaces__tx">{{ formatBytes(total.bytesSent) }} TX</span>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.interfaces {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #ffffff;
}

.interfaces__header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}

.interfaces__header h2 {
  margin: 0;
  font-size: 18px;
}

.interfaces__header p {
  margin: 4px 0 0;
  color: #64748b;
}

.interfaces__count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 28px;
  padding: 4px 9px;
  border-radius: 999px;
  background: #f1f5f9;
  color: #334155;
  font-size: 13px;
  font-weight: 700;
}

.interfaces__empty {
  color: #64748b;
}

.interfaces__list {
  display: grid;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.interfaces__item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 11px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
}

.interfaces__name {
  flex: 1;
  overflow: hidden;
  color: #0f172a;
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.interfaces__rx,
.interfaces__tx {
  flex-shrink: 0;
  color: #475569;
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
</style>
