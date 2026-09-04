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

        <div class="interfaces__metrics">
          <span class="interfaces__metric interfaces__metric--received"
            :title="`Downloaded ${formatBytes(total.bytesReceived)}`">
            <span class="interfaces__arrow" aria-hidden="true">↓</span>
            Received <strong>{{ formatBytes(total.bytesReceived) }}</strong>
          </span>
          <span class="interfaces__metric interfaces__metric--sent" :title="`Uploaded ${formatBytes(total.bytesSent)}`">
            <span class="interfaces__arrow" aria-hidden="true">↑</span>
            Sent <strong>{{ formatBytes(total.bytesSent) }}</strong>
          </span>
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.interfaces {
  min-width: 0;
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
  min-width: 0;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.interfaces__item {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 14px;
  padding: 12px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  background: #ffffff;
}

.interfaces__name {
  overflow: hidden;
  flex: 1 1 auto;
  color: #0f172a;
  font-size: 13px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.interfaces__metrics {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  justify-content: flex-end;
  min-width: 0;
}

.interfaces__metric {
  display: inline-flex;
  align-items: baseline;
  gap: 5px;
  padding: 4px 8px;
  border: 1px solid transparent;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.interfaces__metric strong {
  font-weight: 700;
}

.interfaces__metric--received {
  border-color: #dbeafe;
  background: #eff6ff;
  color: #1d4ed8;
}

.interfaces__metric--received .interfaces__arrow {
  color: #2563eb;
}

.interfaces__metric--sent {
  border-color: #dcfce7;
  background: #f0fdf4;
  color: #15803d;
}

.interfaces__metric--sent .interfaces__arrow {
  color: #16a34a;
}

.interfaces__arrow {
  font-weight: 800;
}

@media (max-width: 1180px) {
  .interfaces__item {
    align-items: flex-start;
    flex-direction: column;
  }

  .interfaces__metrics {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
