<script setup lang="ts">
import { computed } from 'vue'

import type { ScanResult } from '@/types/scan'

const props = defineProps<{
  result: ScanResult
}>()

const disk = computed(() => props.result.diskUsage)

const usedPercentage = computed(() => {
  if (disk.value.totalBytes === 0) {
    return 0
  }

  return Math.min(
    (disk.value.usedBytes / disk.value.totalBytes) * 100,
    100,
  )
})

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
</script>

<template>
  <section class="disk-usage">
    <div class="disk-usage__header">
      <div>
        <h2>Disk Usage</h2>
        <p>{{ disk.path }}</p>
      </div>

      <strong>{{ formatBytes(disk.totalBytes) }}</strong>
    </div>

    <div
      class="disk-usage__track"
      role="progressbar"
      :aria-valuenow="usedPercentage"
      aria-valuemin="0"
      aria-valuemax="100"
      :aria-label="`Disk usage: ${usedPercentage.toFixed(1)}% used`"
    >
      <div
        class="disk-usage__used"
        :style="{ width: `${usedPercentage}%` }"
      />
    </div>

    <div class="disk-usage__stats">
      <div>
        <span>Used</span>
        <strong>{{ formatBytes(disk.usedBytes) }}</strong>
      </div>

      <div>
        <span>Available</span>
        <strong>{{ formatBytes(disk.availableBytes) }}</strong>
      </div>

      <div>
        <span>Free</span>
        <strong>{{ formatBytes(disk.freeBytes) }}</strong>
      </div>
    </div>
  </section>
</template>

<style scoped>
.disk-usage {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
}

.disk-usage__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
}

.disk-usage__header h2 {
  margin: 0;
}

.disk-usage__header p {
  margin: 4px 0 0;
  color: #64748b;
  word-break: break-all;
}

.disk-usage__header > strong {
  font-size: 20px;
}

.disk-usage__track {
  width: 100%;
  height: 18px;
  overflow: hidden;
  border-radius: 999px;
  background: #e2e8f0;
}

.disk-usage__used {
  height: 100%;
  border-radius: inherit;
  background: #0f172a;
  transition: width 400ms ease;
}

.disk-usage__stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-top: 20px;
}

.disk-usage__stats div {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.disk-usage__stats span {
  color: #64748b;
  font-size: 14px;
}

.disk-usage__stats strong {
  font-size: 16px;
}

@media (max-width: 700px) {
  .disk-usage__stats {
    grid-template-columns: 1fr;
  }
}
</style>
