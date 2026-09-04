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
          Ranked from persisted per-minute deltas. Received is traffic your apps downloaded; Sent is
          what they uploaded.
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

        <div class="apps__metrics">
          <span class="apps__chip apps__chip--received"
            :title="`Received (downloaded) ${formatBytes(app.bytesReceived)}`">
            <span class="apps__arrow" aria-hidden="true">↓</span>
            Received
            <strong>{{ formatBytes(app.bytesReceived) }}</strong>
          </span>
          <span class="apps__chip apps__chip--sent" :title="`Sent (uploaded) ${formatBytes(app.bytesSent)}`">
            <span class="apps__arrow" aria-hidden="true">↑</span>
            Sent
            <strong>{{ formatBytes(app.bytesSent) }}</strong>
          </span>
        </div>
      </li>
    </ol>
  </section>
</template>

<style scoped>
.apps {
  min-width: 0;
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #ffffff;
}

.apps__header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}

.apps__header h2 {
  margin: 0;
  font-size: 18px;
}

.apps__header p {
  max-width: 46ch;
  margin: 4px 0 0;
  color: #64748b;
  font-size: 12px;
  line-height: 1.5;
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
  min-width: 0;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.apps__item {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 14px;
  padding: 12px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  background: #ffffff;
}

.apps__rank {
  display: grid;
  width: 26px;
  height: 26px;
  flex: 0 0 26px;
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
  flex: 1 1 auto;
  flex-direction: column;
  gap: 2px;
}

.apps__name {
  overflow: hidden;
  color: #0f172a;
  font-size: 13px;
  font-weight: 600;
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

.apps__metrics {
  display: flex;
  flex: 0 0 auto;
  align-items: flex-end;
  gap: 6px;
  flex-wrap: wrap;
  justify-content: flex-end;
  min-width: 0;
}

.apps__chip {
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

.apps__chip strong {
  font-weight: 700;
}

.apps__chip--received {
  border-color: #dbeafe;
  background: #eff6ff;
  color: #1d4ed8;
}

.apps__chip--received .apps__arrow {
  color: #2563eb;
}

.apps__chip--sent {
  border-color: #dcfce7;
  background: #f0fdf4;
  color: #15803d;
}

.apps__chip--sent .apps__arrow {
  color: #16a34a;
}

.apps__arrow {
  font-weight: 800;
}

@media (max-width: 1180px) {
  .apps__item {
    align-items: flex-start;
    flex-direction: column;
  }

  .apps__identity,
  .apps__metrics {
    width: 100%;
  }

  .apps__metrics {
    justify-content: flex-start;
  }
}
</style>
