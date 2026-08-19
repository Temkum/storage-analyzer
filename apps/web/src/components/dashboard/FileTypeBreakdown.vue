<script setup lang="ts">
import { computed } from 'vue'

import { FileType, type ScanResult } from '@/types/scan'

interface FileTypeSummary {
  extension: string
  count: number
  size: number
}

const props = defineProps<{
  result: ScanResult
}>()

const fileTypes = computed<FileTypeSummary[]>(() => {
  const groups = new Map<string, FileTypeSummary>()

  for (const entry of props.result.entries) {
    if (entry.type !== FileType.File) {
      continue
    }

    const filename = entry.path.split('/').pop() ?? ''
    const dotIndex = filename.lastIndexOf('.')

    const extension =
      dotIndex > 0
        ? filename.slice(dotIndex + 1).toLowerCase()
        : 'no extension'

    const existing = groups.get(extension)

    if (existing) {
      existing.count += 1
      existing.size += entry.size
    } else {
      groups.set(extension, {
        extension,
        count: 1,
        size: entry.size,
      })
    }
  }

  return Array.from(groups.values())
    .slice()
    .sort((a, b) => b.size - a.size)
    .slice(0, 10)
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
    <h2>File Types</h2>

    <p v-if="fileTypes.length === 0">
      No files found.
    </p>

    <div v-else>
      <article
        v-for="fileType in fileTypes"
        :key="fileType.extension"
      >
        <div>
          <strong>
            .{{ fileType.extension }}
          </strong>

          <span>
            {{ fileType.count }} file{{ fileType.count === 1 ? '' : 's' }}
          </span>
        </div>

        <strong>
          {{ formatBytes(fileType.size) }}
        </strong>
      </article>
    </div>
  </section>
</template>
