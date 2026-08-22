<script setup lang="ts">
import { computed } from 'vue'

import { FileType, type ScanResult } from '@/types/scan'
import { formatBytes } from '@/utils/format'

interface StorageSegment {
  label: string
  size: number
  percentage: number
}

const props = defineProps<{
  result: ScanResult
}>()

const totalSize = computed(() => props.result.totalSize)

const segments = computed<StorageSegment[]>(() => {
  if (totalSize.value === 0) {
    return []
  }

  const groups = new Map<string, number>()

  for (const entry of props.result.entries) {
    if (entry.type !== FileType.File) {
      continue
    }

    const filename = entry.path.split('/').pop() ?? ''
    const dotIndex = filename.lastIndexOf('.')

    const extension =
      dotIndex > 0
        ? filename.slice(dotIndex + 1).toLowerCase()
        : 'other'

    groups.set(
      extension,
      (groups.get(extension) ?? 0) + entry.size,
    )
  }

  return Array.from(groups.entries())
    .map(([label, size]) => ({
      label,
      size,
      percentage: Math.min(
        (size / totalSize.value) * 100,
        100,
      ),
    }))
    .sort((a, b) => b.size - a.size)
})
</script>

<template>
  <section class="storage">
    <div class="storage__header">
      <div>
        <h2>Storage Usage</h2>
        <p>Scanned storage grouped by file type.</p>
      </div>

      <strong>{{ formatBytes(totalSize) }}</strong>
    </div>

    <div v-if="segments.length" class="storage__bar" role="img" aria-label="Storage usage by file type">
      <div v-for="segment in segments" :key="segment.label" class="storage__segment"
        :style="{ width: `${segment.percentage}%` }" :title="`${segment.label}: ${formatBytes(segment.size)}`" />
    </div>

    <p v-else>
      No file storage detected.
    </p>

    <div class="storage__legend">
      <div v-for="segment in segments" :key="segment.label" class="storage__item">
        <span>{{ segment.label }}</span>

        <strong>
          {{ formatBytes(segment.size) }}
          ({{ segment.percentage.toFixed(1) }}%)
        </strong>
      </div>
    </div>
  </section>
</template>

<style scoped>
.storage {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
}

.storage__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
}

.storage h2 {
  margin: 0;
}

.storage__header p {
  margin: 4px 0 0;
  color: #64748b;
}

.storage__header>strong {
  font-size: 20px;
}

.storage__bar {
  display: flex;
  width: 100%;
  height: 18px;
  overflow: hidden;
  border-radius: 999px;
  background: #e2e8f0;
}

.storage__segment {
  min-width: 2px;
  transition: width 300ms ease;
}

.storage__segment:nth-child(4n + 1) {
  background: #0f172a;
}

.storage__segment:nth-child(4n + 2) {
  background: #475569;
}

.storage__segment:nth-child(4n + 3) {
  background: #94a3b8;
}

.storage__segment:nth-child(4n + 4) {
  background: #cbd5e1;
}

.storage__legend {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px 24px;
  margin-top: 20px;
}

.storage__item {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 14px;
}

.storage__item span {
  color: #64748b;
}

@media (max-width: 700px) {
  .storage__legend {
    grid-template-columns: 1fr;
  }
}
</style>
