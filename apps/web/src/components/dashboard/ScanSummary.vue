<script setup lang="ts">
import { computed } from 'vue'

import {
  FileType,
  type ScanResult,
} from '@/types/scan'

const props = defineProps<{
  result: ScanResult
}>()

const fileCount = computed(() =>
  props.result.entries.filter(
    (entry) => entry.type === FileType.File,
  ).length,
)

const directoryCount = computed(() =>
  props.result.entries.filter(
    (entry) => entry.type === FileType.Directory,
  ).length,
)

const totalSize = computed(() => {
  if (props.result.directories.length === 0) {
    return 0
  }

  return Math.max(
    ...props.result.directories.map(
      (directory) => directory.size,
    ),
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
