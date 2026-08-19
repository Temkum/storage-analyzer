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
</script>

<template>
  <section>
    <h2>Scan Summary</h2>

    <div>
      <article>
        <span>Total Size </span>
        <strong>{{ formatBytes(result.totalSize) }}</strong>
      </article>

      <article>
        <span>Files </span>
        <strong>{{ result.fileCount }}</strong>
      </article>

      <article>
        <span>Directories </span>
        <strong>{{ result.directoryCount }}</strong>
      </article>

      <article>
        <span>Scan Time </span>
        <strong>{{ result.durationMs }} ms</strong>
      </article>
    </div>
  </section>
</template>
