<script setup lang="ts">
import { computed } from 'vue'

import { FileType, type ScanResult } from '@/types/scan'
import { formatBytes } from '@/utils/format'

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
      dotIndex > 0 ? filename.slice(dotIndex + 1).toLowerCase() : 'no extension'

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

const largestTypeSize = computed(() => {
  const [first] = fileTypes.value

  return first ? first.size : 0
})

function fillWidth(size: number): number {
  if (largestTypeSize.value === 0) {
    return 0
  }

  return (size / largestTypeSize.value) * 100
}
</script>

<template>
  <section class="file-types">
    <div class="file-types__header">
      <h2>File Types</h2>
      <p>Largest extensions by total stored size.</p>
    </div>

    <p v-if="fileTypes.length === 0" class="file-types__empty">
      No files found.
    </p>

    <ul v-else class="file-types__list">
      <li
        v-for="(fileType, index) in fileTypes"
        :key="fileType.extension"
        class="file-types__item"
      >
        <span class="file-types__badge" :class="`file-types__badge--${index % 4}`">
          {{ fileType.extension === 'no extension' ? 'no ext' : `.${fileType.extension}` }}
        </span>

        <div class="file-types__body">
          <div class="file-types__meta">
            <span>
              {{ fileType.count }} file{{ fileType.count === 1 ? '' : 's' }}
            </span>

            <strong>{{ formatBytes(fileType.size) }}</strong>
          </div>

          <div class="file-types__track">
            <div
              class="file-types__fill"
              :style="{ width: `${fillWidth(fileType.size)}%` }"
            />
          </div>
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.file-types {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
}

.file-types__header h2 {
  margin: 0;
  font-size: 18px;
}

.file-types__header p {
  margin: 4px 0 0;
  color: #64748b;
}

.file-types__empty {
  color: #64748b;
}

.file-types__list {
  display: grid;
  gap: 12px;
  margin: 18px 0 0;
  padding: 0;
  list-style: none;
}

.file-types__item {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.file-types__badge {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  min-width: 64px;
  padding: 5px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
}

.file-types__badge--0 {
  background: #0f172a;
  color: #fff;
}

.file-types__badge--1 {
  background: #334155;
  color: #fff;
}

.file-types__badge--2 {
  background: #64748b;
  color: #fff;
}

.file-types__badge--3 {
  background: #e2e8f0;
  color: #334155;
}

.file-types__body {
  min-width: 0;
  flex: 1;
}

.file-types__meta {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
}

.file-types__meta span {
  color: #64748b;
}

.file-types__meta strong {
  color: #0f172a;
  font-size: 13px;
}

.file-types__track {
  height: 6px;
  margin-top: 5px;
  overflow: hidden;
  border-radius: 999px;
  background: #f1f5f9;
}

.file-types__fill {
  height: 100%;
  border-radius: inherit;
  background: #0f172a;
  transition: width 300ms ease;
}
</style>