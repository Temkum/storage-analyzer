<script setup lang="ts">
import { computed } from 'vue'

import { useNetwork } from '@/composables/useNetwork'
import { HISTORY_RANGES } from '@/composables/useNetwork'
import type { HistoryRange } from '@/types/network'
import { formatBytes } from '@/utils/format'

import InterfacesPanel from './InterfacesPanel.vue'
import NetworkOverview from './NetworkOverview.vue'
import TopApplications from './TopApplications.vue'
import ThroughputChart from './ThroughputChart.vue'

const {
  live,
  isConnected,
  currentThroughput,
  range,
  history,
  topApplications,
  isHistoryLoading,
  historyError,
  refreshHistory,
  setRange,
} = useNetwork()

const historyTotals = computed(() => {
  const points = history.value?.totals ?? []

  return points.map((point) => ({
    timestamp: point.ts,
    bytesReceived: point.bytesReceived,
    bytesSent: point.bytesSent,
  }))
})

const historySum = computed(() => {
  const points = history.value?.totals ?? []

  return points.reduce(
    (sum, point) => ({
      bytesReceived: sum.bytesReceived + point.bytesReceived,
      bytesSent: sum.bytesSent + point.bytesSent,
    }),
    { bytesReceived: 0, bytesSent: 0 },
  )
})

function onRangeChange(event: Event) {
  setRange((event.target as HTMLSelectElement).value as HistoryRange)
}
</script>

<template>
  <div class="network">
    <p v-if="!isConnected && live === null" class="network__error" role="alert">
      Network telemetry is unavailable — the sampler could not be reached.
    </p>

    <p v-else-if="live?.error" class="network__warning" role="status">
      Live monitoring is degraded: {{ live.error }} Historical data remains available below.
    </p>

    <section class="network__live panel">
      <div class="network__live-header">
        <div>
          <h2>Network Overview</h2>
          <p>One sampler, one sidecar — always running while the app is open.</p>
        </div>

        <span class="network__status" :class="isConnected ? 'network__status--live' : 'network__status--down'">
          {{ isConnected ? 'Live' : 'Connecting…' }}
        </span>
      </div>

      <NetworkOverview :bytes-received-per-second="currentThroughput.bytesReceived"
        :bytes-sent-per-second="currentThroughput.bytesSent" :connected="isConnected" />

      <ThroughputChart :points="live?.totals ?? []" />
    </section>

    <div class="network__grid">
      <InterfacesPanel :live="live" />
      <TopApplications :applications="topApplications" :loading="isHistoryLoading" />
    </div>

    <section class="network__history panel">
      <div class="network__history-header">
        <div>
          <h2>History</h2>
          <p>
            Persisted per-minute rollups aggregated into
            {{ history?.bucketSeconds ?? 60 }}-second buckets.
          </p>
        </div>

        <label class="network__range">
          <span>Range</span>
          <select :value="range" @change="onRangeChange">
            <option v-for="option in HISTORY_RANGES" :key="option" :value="option">
              Last {{ option }}
            </option>
          </select>
        </label>
      </div>

      <p v-if="historyError" class="network__error" role="alert">
        Failed to load history: {{ historyError }}
        <button type="button" class="network__retry" @click="refreshHistory">Retry</button>
      </p>

      <template v-else>
        <p v-if="isHistoryLoading && history === null" class="network__loading" role="status">
          Loading history…
        </p>

        <div v-else-if="history !== null && history.totals.length === 0" class="network__empty">
          <div class="network__empty-icon">≋</div>
          <h3>No history in this period</h3>
          <p>
            Monitoring persists 60-second rollups while the app runs. This range has no recorded
            traffic yet.
          </p>
        </div>

        <template v-else-if="history !== null">
          <ThroughputChart :points="historyTotals" />

          <div class="network__history-totals">
            <span>
              Total received (↓) <strong>{{ formatBytes(historySum.bytesReceived) }}</strong>
            </span>
            <span>
              Total sent (↑) <strong>{{ formatBytes(historySum.bytesSent) }}</strong>
            </span>
            <span>
              Buckets <strong>{{ history.totals.length }}</strong>
            </span>
          </div>
        </template>
      </template>
    </section>
  </div>
</template>

<style scoped>
.network {
  display: grid;
  gap: 24px;
}

.panel {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #ffffff;
}

.network__error {
  margin: 0;
  padding: 12px 16px;
  border: 1px solid #fecaca;
  border-radius: 10px;
  background: #fef2f2;
  color: #b91c1c;
}

.network__warning {
  margin: 0;
  padding: 12px 16px;
  border: 1px solid #fde68a;
  border-radius: 10px;
  background: #fffbeb;
  color: #92400e;
}

.network__retry {
  margin-left: 10px;
  padding: 3px 10px;
  border: 1px solid #fecaca;
  border-radius: 7px;
  background: #ffffff;
  color: #b91c1c;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.network__live-header,
.network__history-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
}

.network__live-header h2,
.network__history-header h2 {
  margin: 0;
  font-size: 18px;
}

.network__live-header p,
.network__history-header p {
  margin: 4px 0 0;
  color: #64748b;
}

.network__status {
  display: inline-flex;
  align-items: center;
  padding: 5px 11px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
}

.network__status--live {
  background: #dcfce7;
  color: #166534;
}

.network__status--down {
  background: #ffedd5;
  color: #9a3412;
}

.network__grid {
  display: grid;
  gap: 24px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.network__range {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #64748b;
  font-size: 12px;
  font-weight: 700;
}

.network__range select {
  padding: 6px 10px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  background: #ffffff;
  color: #0f172a;
  font-size: 12px;
}

.network__loading {
  color: #64748b;
}

.network__empty {
  display: flex;
  min-height: 220px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border: 1px dashed #cbd5e1;
  border-radius: 12px;
  background: #f8fafc;
  text-align: center;
}

.network__empty-icon {
  display: grid;
  width: 44px;
  height: 44px;
  place-items: center;
  margin-bottom: 12px;
  border-radius: 12px;
  background: #f1f5f9;
  color: #334155;
  font-size: 20px;
}

.network__empty h3 {
  margin: 0;
  color: #0f172a;
  font-size: 15px;
}

.network__empty p {
  max-width: 420px;
  margin: 6px 0 0;
  color: #64748b;
  font-size: 12px;
}

.network__history-totals {
  display: flex;
  gap: 24px;
  margin-top: 14px;
  color: #94a3b8;
  font-size: 12px;
}

.network__history-totals strong {
  color: #334155;
}

@media (max-width: 900px) {
  .network__grid {
    grid-template-columns: 1fr;
  }
}
</style>
