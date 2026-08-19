<script setup lang="ts">
import { computed } from 'vue'

import {
  FileType,
  type FileEntry,
  type ScanResult,
} from '@/types/scan'

const props = defineProps<{
  result: ScanResult
}>()

const largestFiles = computed<FileEntry[]>(() =>
  props.result.entries
    .filter((entry) => entry.type === FileType.File)
    .slice()
    .sort((a, b) => b.size - a.size)
    .slice(0, 10),
)

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
    <h2>Largest Files</h2>

    <p v-if="largestFiles.length === 0">
      No files found.
    </p>

    <ol v-else>
      <li
        v-for="file in largestFiles"
        :key="file.path"
      >
        <span>{{ file.path }}</span>
        <strong>{{ formatBytes(file.size) }}</strong>
      </li>
    </ol>
  </section>
</template>
