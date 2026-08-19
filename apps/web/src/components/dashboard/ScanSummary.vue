<script setup lang="ts">
import { computed } from 'vue'

import type { ScanResult } from '@/types/scan'

const props = defineProps<{
  result: ScanResult
}>()

const fileCount = computed(() =>
  props.result.entries.filter((entry) => entry.type === 0).length,
)

const directoryCount = computed(() =>
  props.result.directories.length,
)

const totalSize = computed(() =>
  props.result.directories.length > 0
    ? props.result.directories[props.result.directories.length - 1].size
    : 0,
)

function formatBytes(bytes: number): string {
  if (bytes === 0) {
    return '0 B'
  }

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const index = Math.floor(Math.log(bytes) / Math.log(1024))

  return `${(bytes / Math.pow(1024, index)).toFixed(2)} ${units[index]}`
}
</script>

<template>
  <section>
    <h2>Scan Summary</h2>

    <div>
      <article>
        <span>Total Size</span>
        <strong>{{ formatBytes(totalSize) }}</strong>
      </article>

      <article>
        <span>Files</span>
        <strong>{{ fileCount }}</strong>
      </article>

      <article>
        <span>Directories</span>
        <strong>{{ directoryCount }}</strong>
      </article>
    </div>
  </section>
</template>
