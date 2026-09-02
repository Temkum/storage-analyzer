<script setup lang="ts">
import type { ApplicationUsage } from '@/types/network'
import { formatBytes } from '@/utils/format'

defineProps<{
  applications: ApplicationUsage[]
  loading: boolean
}>()
</script>

<template>
  <section class="apps">
    <div class="apps__header">
      <div>
        <h2>Top Applications</h2>
        <p>
          Ranked from persisted per-minute deltas. Identity is the executable path — never a PID.
        </p>
      </div>
    </div>

    <p v-if="loading" class="apps__loading" role="status">Loading application usage…</p>

    <p v-else-if="applications.length === 0" class="apps__empty">
      No attributable application traffic in this period. On Linux, only same-UID TCP traffic can be
      attributed — this is not an error.
    </p>

    <ol v-else class="apps__list">
      <li v-for="(app, index) in applications" :key="app.appId" class="apps__item">
        <span class="apps__rank">{{ index + 1 }}</span>

        <div class="apps__identity">
          <strong class="apps__name" :title="app.executablePath ?? app.appId">{{
            app.processName || app.appId
            }}</strong>
          <span class="apps__path" :title="app.appId">{{ app.appId }}</span>
        </div>

        <div class="apps__usage">
          <span class="apps__rx">{{ formatBytes(app.bytesReceived) }} RX</span>
          <span class="apps__tx">{{ formatBytes(app.bytesSent) }} TX</span>
        </div>
      </li>
    </ol>
  </section>
</template>

<style scoped>
.apps {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #ffffff;
}

.apps__header h2 {
  margin: 0;
  font-size: 18px;
}

.apps__header p {
  margin: 4px 0 0;
  color: #64748b;
}

.apps__loading,
.apps__empty {
  color: #64748b;
}

.apps__empty {
  padding: 14px 16px;
  border: 1px dashed #cbd5e1;
  border-radius: 10px;
  background: #f8fafc;
  font-size: 13px;
}

.apps__list {
  display: grid;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
  counter-reset: apps;
}

.apps__item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
}

.apps__rank {
  display: grid;
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  place-items: center;
  border-radius: 999px;
  background: #f1f5f9;
  color: #334155;
  font-size: 11px;
  font-weight: 800;
}

.apps__identity {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 2px;
}

.apps__name {
  overflow: hidden;
  color: #0f172a;
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.apps__path {
  overflow: hidden;
  color: #94a3b8;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.apps__usage {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
}

.apps__rx,
.apps__tx {
  color: #475569;
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
</style>
