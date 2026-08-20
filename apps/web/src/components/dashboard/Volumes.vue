<script setup lang="ts">
import type { ScanResult } from '@/types/scan'

defineProps<{
  result: ScanResult
}>()

function formatBytes(bytes: number): string {
  if (bytes === 0) {
    return '0 B'
  }

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const index = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  )

  return `${(bytes / Math.pow(1024, index)).toFixed(2)} ${units[index]}`
}

function percentage(used: number, total: number): number {
  if (total === 0) {
    return 0
  }

  return Math.min((used / total) * 100, 100)
}
</script>

<template>
  <section class="volumes">
    <div class="volumes__header">
      <div>
        <h2>Volumes</h2>
        <p>Mounted filesystems detected on this system.</p>
      </div>

      <strong>{{ result.volumes.length }}</strong>
    </div>

    <div v-if="result.volumes.length" class="volumes__list">
      <article v-for="volume in result.volumes" :key="volume.mountPoint" class="volumes__item">
        <div class="volumes__meta">
          <div>
            <strong>{{ volume.mountPoint }}</strong>
            <span>{{ volume.filesystem }}</span>
          </div>

          <strong>{{ formatBytes(volume.totalBytes) }}</strong>
        </div>

        <div class="volumes__track">
          <div class="volumes__used" :style="{
            width: `${percentage(volume.usedBytes, volume.totalBytes)}%`,
          }" />
        </div>

        <div class="volumes__stats">
          <span>
            Used {{ formatBytes(volume.usedBytes) }}
          </span>

          <span>
            Available {{ formatBytes(volume.availableBytes) }}
          </span>
        </div>
      </article>
    </div>

    <p v-else>
      No mounted volumes detected.
    </p>
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
}

.volumes__header p {
  margin: 4px 0 0;
  color: #64748b;
}

.volumes__list {
  display: grid;
  gap: 16px;
}

.volumes__item {
  padding: 16px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
}

.volumes__meta {
  display: flex;
  justify-content: space-between;
  gap: 16px;
}

.volumes__meta div {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.volumes__meta span {
  color: #64748b;
  font-size: 13px;
}

.volumes__track {
  height: 10px;
  margin-top: 14px;
  overflow: hidden;
  border-radius: 999px;
  background: #e2e8f0;
}

.volumes__used {
  height: 100%;
  border-radius: inherit;
  background: #0f172a;
  transition: width 400ms ease;
}

.volumes__stats {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  margin-top: 8px;
  color: #64748b;
  font-size: 13px;
}

@media (max-width: 700px) {
  .volumes__stats {
    flex-direction: column;
    gap: 4px;
  }
}
</style>
