<script setup lang="ts">
import { computed } from 'vue'

import type { DirectorySize, ScanResult } from '@/types/scan'

const props = defineProps<{
  result: ScanResult
}>()

const directories = computed<DirectorySize[]>(() =>
  props.result.directories
    .slice()
    .sort((a, b) => b.size - a.size)
    .slice(0, 10),
)

const largestSize = computed(() => {
  const [first] = directories.value

  return first ? first.size : 0
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

function percentage(size: number): number {
  if (largestSize.value === 0) {
    return 0
  }

  return (size / largestSize.value) * 100
}
</script>

<template>
  <section>
    <h2>Directory Breakdown</h2>

    <p v-if="directories.length === 0">
      No directories found.
    </p>

    <div v-else>
      <article
        v-for="directory in directories"
        :key="directory.path"
      >
        <div>
          <span>{{ directory.path }}</span>
          <strong>{{ formatBytes(directory.size) }}</strong>
        </div>

        <div
          role="progressbar"
          :aria-valuenow="directory.size"
          aria-valuemin="0"
          :aria-valuemax="largestSize"
        >
          <div
            :style="{ width: `${percentage(directory.size)}%` }"
          />
        </div>
      </article>
    </div>
  </section>
</template>
