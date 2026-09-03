<script setup lang="ts">
import type { ScanResult } from '@/types/scan'
import { formatBytes } from '@/utils/format'

defineProps<{
  result: ScanResult
}>()

function usagePercentage(used: number, total: number): number {
  if (total === 0) {
    return 0
  }

  return Math.min((used / total) * 100, 100)
}

function usageTone(used: number, total: number): 'normal' | 'warning' | 'critical' {
  const percentage = usagePercentage(used, total)

  if (percentage >= 90) {
    return 'critical'
  }

  if (percentage >= 75) {
    return 'warning'
  }

  return 'normal'
}
</script>

<template>
  <section class="volumes">
    <div class="volumes__header">
      <div>
        <h2>Volumes</h2>
        <p>Mounted filesystems detected on this system.</p>
      </div>

      <span class="volumes__count">{{ result.volumes.length }}</span>
    </div>

    <p v-if="result.volumes.length === 0" class="volumes__empty">
      No mounted volumes detected.
    </p>

    <ul v-else class="volumes__list">
      <li v-for="volume in result.volumes" :key="volume.mountPoint" class="volumes__item">
        <div class="volumes__meta">
          <div class="volumes__identity">
            <strong class="volumes__mount" :title="volume.mountPoint">
              {{ volume.mountPoint }}
            </strong>

            <div class="volumes__badges">
              <span class="volumes__fs">{{ volume.filesystem }}</span>

              <span v-if="volume.readOnly" class="volumes__ro" title="Mounted read-only">
                <svg viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2"
                  stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                  <rect x="3" y="11" width="18" height="11" rx="2" />
                  <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                </svg>
                Read-only
              </span>
            </div>
          </div>

          <span class="volumes__capacity">
            {{ formatBytes(volume.totalBytes) }}
          </span>
        </div>

        <div class="volumes__track" :class="`volumes__track--${usageTone(volume.usedBytes, volume.totalBytes)}`"
          role="progressbar" :aria-valuenow="Math.round(usagePercentage(volume.usedBytes, volume.totalBytes))"
          aria-valuemin="0" aria-valuemax="100"
          :aria-label="`${volume.mountPoint}: ${usagePercentage(volume.usedBytes, volume.totalBytes).toFixed(1)}% used`">
          <div class="volumes__used" :style="{ width: `${usagePercentage(volume.usedBytes, volume.totalBytes)}%` }" />
        </div>

        <div class="volumes__stats">
          <span>
            Used
            <strong>
              {{ formatBytes(volume.usedBytes) }}
              <small>({{ usagePercentage(volume.usedBytes, volume.totalBytes).toFixed(1) }}%)</small>
            </strong>
          </span>

          <span>
            Available
            <strong>{{ formatBytes(volume.availableBytes) }}</strong>
          </span>

          <span>
            Free
            <strong>{{ formatBytes(volume.freeBytes) }}</strong>
          </span>
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.volumes {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
}

.volumes__header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
}

.volumes__header h2 {
  margin: 0;
  font-size: 18px;
}

.volumes__header p {
  margin: 4px 0 0;
  color: #64748b;
}

.volumes__count {
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

.volumes__empty {
  color: #64748b;
}

.volumes__list {
  display: grid;
  max-height: 340px;
  gap: 14px;
  margin: 0;
  padding: 0;
  padding-right: 6px;
  overflow-y: auto;
  list-style: none;
  scrollbar-color: #cbd5e1 transparent;
  scrollbar-width: thin;
}

.volumes__list::-webkit-scrollbar {
  width: 8px;
}

.volumes__list::-webkit-scrollbar-track {
  background: transparent;
}

.volumes__list::-webkit-scrollbar-thumb {
  border: 2px solid transparent;
  border-radius: 999px;
  background: #cbd5e1;
  background-clip: padding-box;
}

.volumes__list::-webkit-scrollbar-thumb:hover {
  background: #94a3b8;
  background-clip: padding-box;
}

.volumes__item {
  padding: 15px 16px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  background: #ffffff;
}

.volumes__meta {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.volumes__identity {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 6px;
}

.volumes__mount {
  overflow: hidden;
  color: #0f172a;
  font-size: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.volumes__badges {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.volumes__fs {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 999px;
  background: #e2e8f0;
  color: #475569;
  font-size: 11px;
  font-weight: 700;
}

.volumes__ro {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 8px;
  border-radius: 999px;
  background: #fef3c7;
  color: #92400e;
  font-size: 11px;
  font-weight: 700;
}

.volumes__capacity {
  flex-shrink: 0;
  color: #0f172a;
  font-size: 13px;
  font-weight: 600;
}

.volumes__track {
  height: 10px;
  margin-top: 14px;
  overflow: hidden;
  border-radius: 999px;
  background: #f1f5f9;
}

.volumes__used {
  height: 100%;
  border-radius: inherit;
  background: #0f172a;
  transition: width 400ms ease;
}

.volumes__track--warning .volumes__used {
  background: #f59e0b;
}

.volumes__track--critical .volumes__used {
  background: #ef4444;
}

.volumes__stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #eef2f7;
  color: #94a3b8;
  font-size: 12px;
}

.volumes__stats span {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.volumes__stats strong {
  color: #334155;
  font-size: 12px;
  font-weight: 600;
}

.volumes__stats strong small {
  color: #94a3b8;
  font-size: 11px;
  font-weight: 600;
}

@media (max-width: 500px) {
  .volumes__stats {
    grid-template-columns: 1fr;
    gap: 8px;
  }
}
</style>
